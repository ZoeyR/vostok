pub trait IntoResponse<Res> {
    fn into_response(self) -> Res;
}

impl<T> IntoResponse<T> for T {
    fn into_response(self) -> T {
        self
    }
}
