extern crate hyper;
extern crate url;
extern crate serialize;
extern crate semver;

use hyper::Client;
use serialize::json;

mod feed;

static DEFAULT_FEED : &'static str = "https://www.myget.org/F/aspnetmaster";

fn main() {
	// Fetch the package list as JSON
	let kre_name = "KRE-CLR-amd64";
	let mut client = Client::new();

	println!("Fetching OData feed...");
	let json = json::from_str(
		client
			.get(format!("{}/Packages?$format=json&$filter=Id eq '{}'", DEFAULT_FEED, kre_name).as_slice())
			.send()
			.unwrap()
			.read_to_string()
			.unwrap()
			.as_slice())
		.unwrap();

	let results = json.find_path(&["d", "results"]).unwrap().as_array().unwrap();
	for result in results.iter() {
		let pkg = feed::Package::from_json(result);
		println!("* {}", pkg);
	}
}
