use crate::data::SysFPrettyContainer;

use super::prelude::*;

use color_eyre::eyre::{self, bail};
use sysf_rs::{
    ast::{core, parse},
    ctx::TyCtxt,
    grammar, pp, pretty, typeck,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[group]
#[prefix("sysf")]
#[commands(parse, resolve, infer)]
/// Various commands doing operations on a System F lambda calculus implemented in Rust.
/// The implementation is available at https://github.com/ThePuzzlemaker/sysf-rs/tree/new-dunfield2013.
/// It is based on the paper "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism".
#[summary = "Bidirectionally typed impredicative System F w/ higher-rank polymorphism"]
pub struct SysF;

async fn int_parse(
    ctx: &Context,
    msg: &Message,
    term: &str,
) -> eyre::Result<(parse::Term, String)> {
    let parser = grammar::TermParser::new();

    let parsed = match parser.parse(term) {
        Ok(p) => p,
        Err(err) => {
            msg.channel_id
                .send_message(&ctx, |b| {
                    b.embed(|e| {
                        begin_error(e)
                            .title("Syntax Error")
                            .description(format!("```\n{}\n```", err))
                    })
                })
                .await?;
            bail!("syfs:parse: Syntax error");
        }
    };

    let pretty = {
        let data = ctx.data.read().await;
        let ppc = data
            .get::<SysFPrettyContainer>()
            .expect("get SysFPrettyContainer");
        ppc.0.send(ToPrettyPrint::ParseTerm(*parsed.clone()))?;
        let mut rx = ppc.1.lock().await;
        rx.recv().await.expect("get pretty-printed")
    };

    Ok((*parsed, pretty))
}

async fn int_resolve(
    ctx: &Context,
    msg: &Message,
    term: parse::Term,
) -> eyre::Result<(core::Term, String)> {
    let core = if let Some(c) = term.into_core() {
        c
    } else {
        msg.channel_id
            .send_message(&ctx, |b| {
                b.embed(|e| {
                    begin_error(e)
                        .title("Resolution Error")
                        .description("(More detail in resolution errors is to come soon.)")
                })
            })
            .await?;
        bail!("sysf:resolve: Resolution error");
    };

    let pretty = {
        let data = ctx.data.read().await;
        let ppc = data
            .get::<SysFPrettyContainer>()
            .expect("get SysFPrettyContainer");
        ppc.0.send(ToPrettyPrint::CoreTerm(core.clone()))?;
        let mut rx = ppc.1.lock().await;
        rx.recv().await.expect("get pretty-printed")
    };

    Ok((core, pretty))
}

#[command]
#[description = "Parse a single term and show the result."]
pub async fn parse(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let term = args.rest().trim_matches('`');

    let (_, pretty) = int_parse(ctx, msg, term).await?;

    msg.channel_id
        .send_message(&ctx, |b| {
            b.embed(|e| {
                begin(e)
                    .title("Parsed Term")
                    .description(format!("```\n{}\n```", pretty))
            })
            .components(|c| build_cleanup(c, msg.author.id))
        })
        .await?;

    Ok(())
}

#[command]
#[description = "Parse and resolve a single term and show the result."]
pub async fn resolve(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let term = args.rest().trim_matches('`');

    let (parsed, pretty_parsed) = int_parse(ctx, msg, term).await?;
    let (_, pretty_core) = int_resolve(ctx, msg, parsed).await?;

    msg.channel_id
        .send_message(&ctx, |b| {
            b.add_embed(move |e| {
                begin(e)
                    .title("Parsed Term")
                    .description(format!("```\n{}\n```", pretty_parsed))
            })
            .add_embed(move |e| {
                begin(e)
                    .title("Resolved Term")
                    .description(format!("```\n{}\n```", pretty_core))
            })
            .components(|c| build_cleanup(c, msg.author.id))
        })
        .await?;

    Ok(())
}

#[command]
#[aliases("typeck", "typecheck", "tc")]
#[description = "Parse, resolve, and typecheck a single term and show the result."]
pub async fn infer(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let term = args.rest().trim_matches('`');

    let (parsed, pretty_parsed) = int_parse(ctx, msg, term).await?;
    let (core, pretty_core) = int_resolve(ctx, msg, parsed).await?;

    let mut tcx = TyCtxt::default();
    let inferred_pretty = match typeck::infer(&mut tcx, &core) {
        Some(ty) => {
            let ty = ty.subst_ctx(&tcx);
            let data = ctx.data.read().await;
            let ppc = data
                .get::<SysFPrettyContainer>()
                .expect("get SysFPrettyContainer");
            ppc.0.send(ToPrettyPrint::CoreTy(ty.clone()))?;
            let mut rx = ppc.1.lock().await;
            format!("```\n{}\n```", rx.recv().await.expect("get pretty-printed"))
        }
        None => "Uninferrable.".to_string(),
    };

    msg.channel_id
        .send_message(&ctx, |b| {
            b.add_embed(move |e| {
                begin(e)
                    .title("Parsed Term")
                    .description(format!("```\n{}\n```", pretty_parsed))
            })
            .add_embed(move |e| {
                begin(e)
                    .title("Resolved Term")
                    .description(format!("```\n{}\n```", pretty_core))
            })
            .add_embed(move |e| begin(e).title("Inferred Type").description(inferred_pretty))
            .add_embed(move |e| {
                begin(e)
                    .title("TyCtxt")
                    .description(format!("```\n{:?}\n```", tcx))
            })
            .components(|c| build_cleanup(c, msg.author.id))
        })
        .await?;

    Ok(())
}

#[derive(Clone, Debug)]
pub enum ToPrettyPrint {
    ParseTerm(parse::Term),
    CoreTerm(core::Term),
    CoreTy(core::Ty),
}

pub async fn pretty_printing(
    tx: UnboundedSender<String>,
    mut rx: UnboundedReceiver<ToPrettyPrint>,
) {
    let arena = pretty::Arena::new();

    while let Some(to_pp) = rx.recv().await {
        let pp = match to_pp {
            ToPrettyPrint::ParseTerm(term) => pp::pp_parse_term(term, &arena),
            ToPrettyPrint::CoreTerm(term) => pp::pp_core_term(term, &arena),
            ToPrettyPrint::CoreTy(ty) => pp::pp_core_ty(ty, &arena),
        };

        tx.send(format!("{}", pp.into_doc().pretty(80)))
            .expect("send pretty-printed");
    }
}
