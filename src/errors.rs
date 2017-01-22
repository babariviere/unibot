
error_chain!{
    foreign_links {
        Url(::hyper::error::ParseError);
    }
}
