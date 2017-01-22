
error_chain!{
    foreign_links {
        Url(::url::ParseError);
    }
}
