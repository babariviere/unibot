
error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        Url(::hyper::error::ParseError);
    }

    errors {
        UrlAlreadyIndexed {
            description("Url is already indexed")
            display("Could not add url because it is already indexed")
        }
    }
}
