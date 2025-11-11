use polypathroute_core::CoreContext;

mod adapters;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let core_context = CoreContext::new("./src/config/config.toml");
        // println!("{:?}", core_context);

        let stargate_adapter = adapters::create_adapter("stargate").unwrap();
        println!("fetch_metrics: {:?}", stargate_adapter.fetch_metrics("ethereum", "polygon", "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359", "1000000", "990000", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a").unwrap());
    }
}


