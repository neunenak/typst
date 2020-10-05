use crate::prelude::*;

/// `align`: Align content along the layouting axes.
///
/// # Positional arguments
/// - At most two of `left`, `right`, `top`, `bottom`, `center`.
///
/// When `center` is used as a positional argument, it is automatically inferred
/// which axis it should apply to depending on further arguments, defaulting
/// to the axis, text is set along.
///
/// # Keyword arguments
/// - `horizontal`: Any of `left`, `right` or `center`.
/// - `vertical`: Any of `top`, `bottom` or `center`.
///
/// There may not be two alignment specifications for the same axis.
pub async fn align(mut args: Args, ctx: &mut LayoutContext) -> Value {
    let body = args.find::<SynTree>();
    let first = args.get::<_, Spanned<SpecAlign>>(ctx, 0);
    let second = args.get::<_, Spanned<SpecAlign>>(ctx, 1);
    let hor = args.get::<_, Spanned<SpecAlign>>(ctx, "horizontal");
    let ver = args.get::<_, Spanned<SpecAlign>>(ctx, "vertical");
    args.done(ctx);

    let iter = first
        .into_iter()
        .chain(second.into_iter())
        .map(|align| (align.v.axis(), align))
        .chain(hor.into_iter().map(|align| (Some(SpecAxis::Horizontal), align)))
        .chain(ver.into_iter().map(|align| (Some(SpecAxis::Vertical), align)));

    let aligns = dedup_aligns(ctx, iter);

    Value::Commands(match body {
        Some(tree) => vec![
            SetAlignment(aligns),
            LayoutSyntaxTree(tree),
            SetAlignment(ctx.state.align),
        ],
        None => vec![SetAlignment(aligns)],
    })
}

/// Deduplicate alignments and deduce to which axes they apply.
fn dedup_aligns(
    ctx: &mut LayoutContext,
    iter: impl Iterator<Item = (Option<SpecAxis>, Spanned<SpecAlign>)>,
) -> LayoutAlign {
    let mut aligns = ctx.state.align;
    let mut had = Gen2::new(false, false);
    let mut had_center = false;

    for (axis, Spanned { v: align, span }) in iter {
        // Check whether we know which axis this alignment belongs to.
        if let Some(axis) = axis {
            // We know the axis.
            let gen_axis = axis.to_gen(ctx.state.sys);
            let gen_align = align.to_gen(ctx.state.sys);

            if align.axis().map_or(false, |a| a != axis) {
                ctx.diag(error!(
                    span,
                    "invalid alignment `{}` for {} axis", align, axis,
                ));
            } else if had.get(gen_axis) {
                ctx.diag(error!(span, "duplicate alignment for {} axis", axis));
            } else {
                *aligns.get_mut(gen_axis) = gen_align;
                *had.get_mut(gen_axis) = true;
            }
        } else {
            // We don't know the axis: This has to be a `center` alignment for a
            // positional argument.
            debug_assert_eq!(align, SpecAlign::Center);

            if had.primary && had.secondary {
                ctx.diag(error!(span, "duplicate alignment"));
            } else if had_center {
                // Both this and the previous one are unspecified `center`
                // alignments. Both axes should be centered.
                aligns = LayoutAlign::new(GenAlign::Center, GenAlign::Center);
                had.primary = true;
                had.secondary = true;
            } else {
                had_center = true;
            }
        }

        // If we we know one alignment, we can handle the unspecified `center`
        // alignment.
        if had_center && (had.primary || had.secondary) {
            if had.primary {
                aligns.secondary = GenAlign::Center;
                had.secondary = true;
            } else {
                aligns.primary = GenAlign::Center;
                had.primary = true;
            }
            had_center = false;
        }
    }

    // If center has not been flushed by now, it is the only argument and then
    // we default to applying it to the primary axis.
    if had_center {
        aligns.primary = GenAlign::Center;
    }

    aligns
}
