use std::error::{Error, FromError};

use semver;
use semver::Version;

use hyper::Url;

use url;

use serialize::json::Json;

#[derive(Show)]
pub enum PackageRuntime {
	CLR,
	CoreCLR,
	Mono
}

#[derive(Show)]
pub enum PackageArchitecture {
	x86,
	amd64
}

#[derive(Show)]
pub struct Package {
	id: String,
	version: Version,
	runtime: PackageRuntime,
	architecture: PackageArchitecture,
	url: Url,
	download_url: Url
}

#[derive(Show)]
pub struct PackageParseError {
	field: String,
	kind: FieldErrorKind
}

impl PackageParseError {
	fn new(field: String, kind: FieldErrorKind) -> PackageParseError {
		PackageParseError {
			field: field,
			kind: kind
		}
	}
}

#[derive(Show)]
pub enum FieldErrorKind {
	Missing,
	IncorrectType,
	InvalidVersion(semver::ParseError),
	InvalidPackageId,
	InvalidUrl(url::ParseError)
}

impl Package {
	fn from_json(json: &Json) -> Result<Package, PackageParseError> {
		let id = try!(get_json_str(json, &["Id"]));
		let ver = try!(get_json(json, &["NormalizedVersion"], |s| Version::parse(s)));
		let (runtime, architecture) = try!(parse_id(id));
		let url = try!(get_json(json, &["__metadata", "uri"], |s| Url::parse(s)));
		let download_url = try!(get_json(json, &["__metadata", "media_src"], |s| Url::parse(s)));

		Ok(Package {
			id: id.to_string(),
			version: ver,
			runtime: runtime,
			architecture: architecture,
			url: url,
			download_url: download_url
		})
	}
}

fn parse_id(id: &str) -> Result<(PackageRuntime, PackageArchitecture), PackageParseError> {
	let fields : Vec<&str> = id.split('-').collect();
	match fields.as_slice() {
		["KRE", runtime, architecture] => Ok((
			match runtime {
				"CLR" => PackageRuntime::CLR,
				"CoreCLR" => PackageRuntime::CoreCLR,
				"Mono" => PackageRuntime::Mono
			}, 
			match architecture {
				"x86" => PackageArchitecture::x86,
				"amd64" => PackageArchitecture::amd64
			})),
		_ => Err(PackageParseError::new("Id".to_string(), FieldErrorKind::InvalidPackageId))
	}
}

fn get_json_str<'a>(json: &'a Json, keys: &[&str]) -> Result<&'a str, PackageParseError> {
	let full_key = keys.iter().fold(String::new(), |a, &b| a + b + ".");
	full_key.truncate(full_key.len() - 1);

	let prop = try!(json.find_path(keys).ok_or(PackageParseError::new(full_key, FieldErrorKind::Missing)));
	let value = try!(prop.as_string().ok_or(PackageParseError::new(full_key, FieldErrorKind::IncorrectType)));

	Ok(value)
}

fn get_json<T, E, P: FromError<E>, O: FnOnce(&str) -> Result<T, E>>(json: &Json, keys: &[&str], converter: O) -> Result<T, P> {
	try!(converter(try!(get_json_str(json, keys))))
}