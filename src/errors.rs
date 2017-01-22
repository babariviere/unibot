
error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        Url(::hyper::error::ParseError);
    }
}
