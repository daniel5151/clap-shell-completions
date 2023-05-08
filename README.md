# clap-shell-completions

A hand-rolled version of "clap_complete", with support for dynamic completions
driven by callbacks.

The code ain't polished, ain't tested, and ain't particularly pretty. There's
still plenty to do.

> NOTE: It was only after spending a few hours hacking on this that I realized
> <https://github.com/clap-rs/clap/issues/3166> was already a thing. Oops.
>
> Then again, that issue has been open for over a year, and seems to be moving
> slowly, and I needed something that works today, soooo...

## Inspiration

Rather than emitting a big 'ol blob of shell-specific completion code, move all
the "heavy lifting" into a single Rust implementation that outputs a list of
completions, and then wrap that in a tiny bit of shell-specific glue code.

This is actually what `dotnet` does for its completions:
<https://learn.microsoft.com/en-us/dotnet/core/tools/enable-tab-autocomplete>
