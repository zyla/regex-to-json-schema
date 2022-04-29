use regex_syntax::hir::{Hir, HirKind, Literal};

fn main() {
    let hir = regex_syntax::Parser::new().parse("a(bc|bd)*(e|f)").unwrap();
    let mut nfa = Nfa::default();
    let start = nfa.new_state();
    let end = regex_to_nfa(&mut nfa, &hir, start);
    print_dot(&nfa, start, end);
}

fn print_dot(nfa: &Nfa, start: State, end: State) {
    println!("digraph {{");
    println!("rankdir=LR");
    println!("\"\" [shape=none]");
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
                Transition::Goto(to) => println!("{} -> {}", from, to),
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
    fn new_state(&mut self) -> State {
        let s = self.states.len();
        self.states.push(Default::default());
        s
    }

    fn add_transition(&mut self, from: State, t: Transition) {
        self.states[from].push(t);
    }
}

fn regex_to_nfa(nfa: &mut Nfa, r: &Hir, mut start: State) -> State {
    match r.kind() {
        HirKind::Empty => start,
        HirKind::Class(_) => unimplemented!("character classes not supported"),
        HirKind::Group(g) => regex_to_nfa(nfa, &g.hir, start),
        HirKind::Anchor(_) => unimplemented!("anchors not supported"),
        HirKind::Concat(xs) => {
            for x in xs {
                start = regex_to_nfa(nfa, x, start);
            }
            start
        }
        HirKind::Literal(lit) => {
            let next = nfa.new_state();
            let c = match lit {
                Literal::Unicode(c) => *c,
                Literal::Byte(b) => *b as char,
            };
            nfa.add_transition(start, Transition::Consume(c, next));
            next
        }
        HirKind::Repetition(rep) => {
            // TODO: repetition types
            let next = regex_to_nfa(nfa, &rep.hir, start);
            nfa.add_transition(next, Transition::Goto(start));
            next
        }
        HirKind::Alternation(branches) => {
            let end = nfa.new_state();
            for branch in branches {
                let next = regex_to_nfa(nfa, branch, start);
                nfa.add_transition(next, Transition::Goto(end));
            }
            end
        }
        HirKind::WordBoundary(_) => unimplemented!("word boundary not supported"),
    }
}
