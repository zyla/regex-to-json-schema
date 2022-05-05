use clap::{Parser, Subcommand};
use regex_syntax::hir::{Hir, HirKind, Literal, RepetitionKind};
use std::collections::VecDeque;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert the regular expression to NFA, and output it in DOT format.
    Nfa { regex: String },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Nfa { regex } => {
            let hir = regex_syntax::Parser::new().parse(&regex).unwrap();
            let mut nfa = Nfa::default();
            let start = nfa.new_state();
            let end = nfa.new_state();
            regex_to_nfa(&mut nfa, &hir, start, end);
            let state_mapping = renumber_states(&nfa, start);
            print_dot(&nfa, &state_mapping, start, end);
        }
    }
}

fn print_dot(nfa: &Nfa, state_mapping: &[State], start: State, end: State) {
    println!("digraph {{");
    println!("rankdir=LR");
    println!("\"\" [shape=none]");
    for state in 0..nfa.num_states() {
        println!("{} [label=\"{}\"]", state, state_mapping[state]);
    }
    println!("\"\" -> {}", start);
    for (from, transitions) in nfa.states.iter().enumerate() {
        println!(
            "{} [shape={}]",
            from,
            if from == end {
                "doublecircle"
            } else {
                "circle"
            }
        );
        for t in transitions {
            match t {
                Transition::Goto(to) => println!("{} -> {} [label=\" \"]", from, to),
                Transition::Consume(input, to) => {
                    println!("{} -> {} [label=\"{}\"]", from, to, input)
                }
            }
        }
    }
    println!("}}");
}

type State = usize;

#[derive(Debug)]
enum Transition {
    Goto(State),
    Consume(char, State),
}

#[derive(Debug)]
struct Nfa {
    states: Vec<Vec<Transition>>,
}

impl Default for Nfa {
    fn default() -> Self {
        Self {
            states: Default::default(),
        }
    }
}

impl Nfa {
    fn num_states(&self) -> usize {
        self.states.len()
    }

    fn new_state(&mut self) -> State {
        let s = self.states.len();
        self.states.push(Default::default());
        s
    }

    fn add_transition(&mut self, from: State, t: Transition) {
        self.states[from].push(t);
    }
}

fn regex_to_nfa(nfa: &mut Nfa, r: &Hir, mut start: State, end: State) {
    match r.kind() {
        HirKind::Empty => nfa.add_transition(start, Transition::Goto(end)),
        HirKind::Class(_) => unimplemented!("character classes not supported"),
        HirKind::Group(g) => regex_to_nfa(nfa, &g.hir, start, end),
        HirKind::Anchor(_) => unimplemented!("anchors not supported"),
        HirKind::Concat(xs) => {
            for (i, x) in xs.iter().enumerate() {
                let next = if i == xs.len() - 1 {
                    end
                } else {
                    nfa.new_state()
                };
                regex_to_nfa(nfa, x, start, next);
                start = next;
            }
        }
        HirKind::Literal(lit) => {
            let c = match lit {
                Literal::Unicode(c) => *c,
                Literal::Byte(b) => *b as char,
            };
            nfa.add_transition(start, Transition::Consume(c, end));
        }
        HirKind::Repetition(rep) => match rep.kind {
            RepetitionKind::ZeroOrOne => {
                regex_to_nfa(nfa, &rep.hir, start, end);
                nfa.add_transition(start, Transition::Goto(end));
            }
            RepetitionKind::ZeroOrMore => {
                regex_to_nfa(nfa, &rep.hir, start, start);
                nfa.add_transition(start, Transition::Goto(end));
            }
            RepetitionKind::OneOrMore => {
                regex_to_nfa(nfa, &rep.hir, start, end);
                nfa.add_transition(end, Transition::Goto(start));
            }
            RepetitionKind::Range(_) => unimplemented!(),
        },
        HirKind::Alternation(branches) => {
            for branch in branches {
                regex_to_nfa(nfa, branch, start, end);
            }
        }
        HirKind::WordBoundary(_) => unimplemented!("word boundary not supported"),
    }
}

fn renumber_states(nfa: &Nfa, start: State) -> Vec<State> {
    let mut state_mapping = vec![0; nfa.num_states()];
    let mut next_id = 0;
    let mut queued = vec![false; nfa.num_states()];
    let mut queue = VecDeque::new();
    queue.push_back(start);
    queued[start] = true;
    while let Some(state) = queue.pop_front() {
        let id = next_id;
        next_id += 1;
        state_mapping[state] = id;
        for t in &nfa.states[state] {
            let target = *match t {
                Transition::Consume(_, target) => target,
                Transition::Goto(target) => target,
            };
            if queued[target] {
                continue;
            }
            queue.push_back(target);
            queued[target] = true;
        }
    }
    state_mapping
}
