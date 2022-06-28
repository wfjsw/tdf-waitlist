use rocket::http::Status;
use rocket::response::Responder;
use rocket::Response;
use rocket::Request;

#[derive(Debug)]
pub struct InternalRedirect {
    redirect_url: String,
}

impl InternalRedirect {
    pub fn url(data: String) -> Self {
        Self {
            redirect_url: data,
        }
    }
}

impl<'a> Responder<'a, 'static> for InternalRedirect {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        Response::build() 
            .raw_header("X-Accel-Redirect", self.redirect_url)
            .ok()
    }
}
