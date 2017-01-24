
error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        Url(::hyper::error::ParseError);
    }

    errors {
        UrlAlreadyIndexed {
            description("Could not add url because it is already indexed")
            display("Url is already indexed")
        }
        SpiderTrap {
            description("Site contains spider trap, this is error help to avoid loop")
            display("Site contains spider trap")
        }
    }
}
