use polypathroute_core::CoreContext;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let core_context = CoreContext::new("./src/config/config.toml");
        println!("{:?}", core_context);
    }
}
