use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SerNamedTuple<'a, 'b, A: 'a, B: 'b, C>(&'a A, &'b mut B, C);

#[derive(Deserialize)]
struct DeNamedTuple<A, B, C>(A, B, C);
