use std::io::{self, IsTerminal, Stdin, Stdout, Write};

pub enum PlacementDescriptor
{
    /// First in pipe writes to stdout
    First(Stdout),

    /// Middle in pipe reads from stdin and writes to stdout
    Middle(Stdin, Stdout),

    /// Last in pipe reads from stdin
    Last(Stdout),

    /// Standalone writes to stdout usually
    Alone(Stdout),
}

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

impl From<PlacementDescriptor> for PipelineInvocation
{
    fn from(placement: PlacementDescriptor) -> Self
    {
        use PlacementDescriptor::*;

        match placement
        {
            First(stdout) => Self::PipeOut(stdout),
            Middle(stdin, stdout) => Self::PipeInOut(stdin, stdout),
            Last(..) => Self::PipeIn(std::io::stdin()),
            Alone(..) => Self::NoPipe,
        }
    }
}

impl From<PipelineInvocation> for PlacementDescriptor
{
    fn from(placement: PipelineInvocation) -> Self
    {
        use PipelineInvocation::*;

        match placement
        {
            PipeIn(..) => Self::Last(std::io::stdout()),
            PipeOut(stdout) => Self::First(stdout),
            PipeInOut(stdin, stdout) => Self::Middle(stdin, stdout),
            NoPipe => Self::Alone(std::io::stdout()),
        }
    }
}

impl PipelineInvocation
{
    pub fn placement(&self) -> PlacementDescriptor
    {
        panic!();
    }

    pub fn get() -> Self
    {
        let stdin_piped = !std::io::stdin().is_terminal();
        let stdout_piped = !std::io::stdout().is_terminal();

        let stdin = std::io::stdin();
        let stdout = std::io::stdout();

        // if !stdin_piped && stdout_piped
        // {
        //     // First in pipe: SELF | A0 | B0
        //     PipelineInvocation::PipeOut(std::io::stdout())
        // }
        // else if stdin_piped && stdout_piped
        // {
        //     // Middle in pipe: A0 | SELF | B0
        //     PipelineInvocation::PipeInOut(std::io::stdin(), std::io::stdout())
        // }
        // else if stdin_piped && !stdout_piped
        // {
        //     // Last in pipe: A0 | B0 | SELF
        //     PipelineInvocation::PipeIn(std::io::stdin())
        // }
        // else
        // {
        //     PipelineInvocation::NoPipe
        // }

        // ? More efficient, more clear? Truth table?
        match (stdin_piped, stdout_piped)
        {
            (false, true) => PipelineInvocation::PipeOut(stdout),
            (true, true) => PipelineInvocation::PipeInOut(stdin, stdout),
            (true, false) => PipelineInvocation::PipeIn(stdin),
            (false, false) => PipelineInvocation::NoPipe,
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
    eprintln!("Invoked within pipeline:\n    {:?}", PipelineInvocation::get());
}
