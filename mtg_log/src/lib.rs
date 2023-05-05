use mtg_log_macro::MTGLoggable;

#[derive(MTGLoggable)]
struct Test{
    x:i32,
}

#[derive(MTGLoggable)]
struct Unnamed(usize);


#[derive(MTGLoggable)]
struct Unit;
#[cfg(test)]
mod tests {
    use super::*;
}
