// - typed "some input"
// 	write cli input to stderr to show it
// - typed "some input" | typed
// 	write stdin to stderr to show it
// - typed "some input" | typed
// 	prints stdin to stdout
// - typed "some input" | typed | typed
// 	read stdin
// 	write stdin to stderr to show it
// 	write stdin to stdout to move it
// 	read stdin
// 	write stdin to stderr to show it

use std::io::{IsTerminal, Stdin, Stdout};

pub enum PipelineInvocation
{
    /// Last in pipe: `$ A0 | B0 | SELF`
    PipeIn(Stdin),

    /// First in pipe: `$ SELF | A0 | B0`
    PipeOut(Stdout),

    /// Middle in pipe: `$ A0 | SELF | B0`
    PipeInOut(Stdin, Stdout),

    /// Not in pipeline: `$ SELF`
    NoPipe,
}

// read-from write-to

impl PipelineInvocation
{
    pub fn get() -> Self
    {
        let stdin_piped = !std::io::stdin().is_terminal();
        let stdout_piped = !std::io::stdout().is_terminal();

        if !stdin_piped && stdout_piped
        {
            // First in pipe: SELF | A0 | B0
            PipelineInvocation::PipeOut(std::io::stdout())
        }
        else if stdin_piped && stdout_piped
        {
            // Middle in pipe: A0 | SELF | B0
            PipelineInvocation::PipeInOut(std::io::stdin(), std::io::stdout())
        }
        else if stdin_piped && !stdout_piped
        {
            // Last in pipe: A0 | B0 | SELF
            PipelineInvocation::PipeIn(std::io::stdin())
        }
        else
        {
            PipelineInvocation::NoPipe
        }
    }
}

impl std::fmt::Debug for PipelineInvocation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::PipeIn(_) => write!(f, "<PipeIn: A0 | SELF>"),
            Self::PipeOut(_) => write!(f, "<PipeOut: SELF | A0>"),
            Self::PipeInOut(..) => write!(f, "<PipeInOut: A0 | SELF | B0>"),
            Self::NoPipe => write!(f, "<NoPipe>"),
        }
    }
}

/// Test program for typed pipes
fn main()
{
    println!("Invoked within pipeline:\n    {:?}", PipelineInvocation::get());
}
