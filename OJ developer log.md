# References

1. [What is a Monitor in CS?](https://www.baeldung.com/cs/monitor)

   Monitor -- a synchronization mechanism

   It allows the threads to have:

   - mutual exclusion, i. e. only one thread can execute the method at a certain point in time, using *locks*
   - cooperation,  i. e. make threads wait for certain conditions to be met, using *wait-set*

   It gets its name "Monitor" as it monitors how threads access some resources.

   One thread can notify other threads when conditions they're waiting on are met.

2. [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)

   Asynchronous Programming is a *concurrent programming model*, it lets you run a large number of concurrent tasks on a small number of OS threads, while preserving much of the look and feel of ordinary synchronous programming, through the `async/await` syntax.

3. [How to assign a value to each field in the struct in the declaration of the fields](https://stackoverflow.com/questions/19650265/is-there-a-faster-shorter-way-to-initialize-variables-in-a-rust-struct)

4. [Error: cannot find derive macro Deserialize in this scope](https://github.com/serde-rs/serde/issues/1586)

   [How to implement `Serialize` and `Deserialize` trait manually](https://serde.rs/custom-serialization.html)

5. [VSCode REST Client --  Header name must be a valid HTTP token ["{"]](https://github.com/Huachao/vscode-restclient/issues/430)

6. VSCode REST Client --  Connection is being rejected for localhost

   https://github.com/Huachao/vscode-restclient/issues/523

   https://stackoverflow.com/questions/64828363/rest-client-proxy-issue-in-vscode

7. [can not move out of an `Arc`](https://stackoverflow.com/questions/69384070/cannot-move-out-of-an-arc)

   use `&self` instead of `self`

8. [wall time (real-world time or wall-clock time)](https://www.techtarget.com/whatis/definition/wall-time-real-world-time-or-wall-clock-time)

9. [How to change the real time into String?](https://stackoverflow.com/questions/71277716/how-to-convert-instant-time-to-string)

   `use chrono::Utc`

10. [How to convert Vec<char> to a string](https://stackoverflow.com/questions/23430735/how-to-convert-vecchar-to-a-string)

11. [How to kill the process?](https://askubuntu.com/questions/104903/how-do-i-kill-processes-in-ubuntu)

    `kill -15 -1`

12. [How to change the permission for a program](https://stackoverflow.com/questions/18960689/running-my-program-says-bash-program-permission-denied)

13. 




# API

`API` is consisted of two parts:

- `GET`: HTTP GET request, transfer the arguments through URL, used to get data

  **Rrmark:** URL, i. e. Uniform Resource Locator, is used to locate a resource on the Internet. It is also referred to as a web address.URLs consist of multiple parts -- including a protocol and domain name -- that tell a web browser how and where to retrive a resource.

- `POST/PUT/DELETE`: HTTP POST/PUT/DELETE requests, arguments are transferred as JSON format.

## About the Judge Task

`POST/jobs`:

- submit the code and create a new judge task
- the request contains message body in JSON format
- After OJ received the request, it will do some checks at first
- If the task is created successfully, return the HTTP condition code and the Message Body in JSON format

`GET/jobs`:

- give the URL, query and filtrate the tasks
- return the results in JSON

`GET/jobs/{jobId}`:

- query the single task

`PUT/jobs/{jobId}`

`DELETE/jobs/{jobId}`:

- delete the judge task

## About the User

`POST/users`:

- create new users or update the user
- use JSON format

`GET/users`:

- get the users list

## About the contests

`POST/contests`:

- create new contests or update

`GET/contests`:

- get the contests list

`GET/contests/{contestId}`

`GET/contests/{contestId}/ranklist`:

- calculate the ranklist



# Configuration

configuration format:

- OJ system uses a single local JSON file to configure
- 





# Syntax

## actix-web

[actix-web official docs](https://actix.rs/docs/getting-started/)

1. Request handlers use `async` functions that accept parameters. These parameters can be extracted from a request (`FromRequest` Trait), returns a type that can be converted into an `HttpResponse` (`Responder` Trait).

2. Use `App::service` for the handlers using routing macros and `App::route` for manually routed handlers, declaring the path and method. The app is started inside an `HttpServer` which will serve incoming requests using `App` as an "application factory".

3. All `actix-web` servers are built around the `App` instance. 

   Application state is shared with all routes and resources within the same scope. State can be accessed with the `web::Data<T>` extractor where `T` is the type of the state.

   ```rust
   use actix_web::{get, web, App, HttpServer};
   
   // This struct represents state
   struct AppState {
       app_name: String
   }
   
   #[get("/")]
   async fn index(data: web::Data<AppState>) -> String {
       let app_name = &data.app_name; // <- get app_name
       format!("Hello {app_name}!") // <- response with app_name
   }
   ```

   Pass in the state when initializing the App and start the application.

4. Shared Mutable State

   `HttpServer` accepts an application factory rather than an application instance, it constructs an application instance for each thread. If you want to share data between different threads, a shareable object should be used, e.g. `Send` + `Sync`.

   First, define a state and create the handler:

   ```rust
   use actix_web::{web, App, HttpServer};
   use std::sync::Mutex;
   
   struct AppStateWithCounter {
       counter: Mutex<i32> // <- Mutex is necessary to mutalbe safely across threads
   }
   
   async fn index(data: web::Data<AppStateWithCounter>) -> String {
       let mut counter = data.counter.lock().unwrap(); // <- counter's MutexGuard
       *counter += 1; // <- access counter inside MutexGuard
       
       format!("Request number: {counter}!") // <- response with count
   }
   ```

   Then register the data in an `App`:

   ```rust
   #[actix_web::main]
   async fn main() -> std::io::Result<()> {
       // Note: web::Data created _outside_ HttpServer::new closure
       let counter = web::Data::new(AppStateWithCounter {
           counter: Mutex::new(0)
       });
       
       HttpServer::new(move || {
           // move counter into the closure
           App::new()
           	.app_data(counter.clone()) // <- register the created data
           	.route("/", web::get().to(index))
       })
       .blind(("127.0.0.1", 8080))?
       .run()
       .await
   }
   ```

   **Remark:**

   > State initialized *inside* the closure passed to `HttpServer::new` is local to the worker thread and may become de-syncd if modified.
   >
   > To achieve *globally shared state*, it must be created **outside** of the closure passed to `HttpServer::new` and moved/cloned in.

5. Using an Application Scope to Compose Applications

   The `web::scope()` method allows setting a resource group prefix. This scope represents a resource prefix that will be prepended to all resource patterns added by the resource configuration.

   ```rust
   #[actix_web::main]
   async fn main()
   {
       let scope = web::scope("/users").service(show_users);
       App::new().service(scope);
   }
   ```

   Then the route will then only match if the URL path is `/users/show` rather than `/show`.

6. Application guards and virtual hosting

   Guard is a simple function that accepts a *request* object reference and returns *true* or *false*. Formally, a guard is any object that implements the `Guard` trait.

   One of the provided guards is `Header`. It can be used as a filter based on request header information.

   ```rust
   #[actix_web::main]
   async fn main() -> std::io::Result<()> {
       HttpServer::new(|| {
           App::new()
           	.service(
           		web::scope("/")
           			.guard(guard::Header("Host", "www.rust-lang.org"))
           			.route("", web::to(|| async { HttpResponse::Ok().body("www") })),
           )
           .service(
           	web::scope("/")
           		.guard(guard::Header("Host", "user.rust-lang.org"))
           		.route("", web::to(|| async { HttpResponse::Ok().body("user") })),
           )
           .route("/", web::to(HttpResponse::Ok))
       })
       .bind(("127.0.0.1", 8080))?
       .run()
       .await
   }
   ```

7. Configure

   Both `App` and `web::Scope` provide the `configure` method. This function is useful for moving parts of the configuration to a different module or library. 
   
   [How to register the data in an `App`:](https://actix.rs/docs/application/)
   
   ```rust
   let config = get_config();
   
       HttpServer::new(move || {
           App::new()
               .wrap(Logger::default())
               .route("/hello", web::get().to(|| async { "Hello World!" }))
               .service(greet)
               // DO NOT REMOVE: used in automatic testing
               .service(exit)
               .service(post_jobs)
               .app_data(web::Data::new(config.clone()))
   
       })
       .bind(("127.0.0.1", 12345))?
       .run()
       .await
   ```
   
   

## Command arguments

```rust
for arg in std::env::args() {}
```

## JSON

How to get the struct from a JSON file?

```rust
let mut json_record = {
	let json_record = std::fs::read_to_string(&address);
    
    // Load the State structure from the string.
    let s: String = json_record.unwrap();
    //println!("{:#?}", s);
    if s.clone() == "{}" {
    	State {
        	total_rounds: 0,
            games: Vec::new()
        }
    } else {
        serde_json::from_str::<State>(&s).unwrap()
    }
};
```

## std::process::Command

A process builder, providing fine-grained control over how a new process should be spwaned.

Builder methods allow the configuration to be changed prior to spwaning.

```rust
use std::progress::Command;

let output = if cfg!(target_os = "windows") {
    Command::new("cmd")
    	.args(["/C", "echo hello"])
    	.output()
    	.expect("failed to execute process")
} else {
    Command::new("sh")
    	.arg("-c")
    	.arg("echo hello")
    	.output()
    	.expect("failed to execute process")
};

let hello = output.stdout;
```

`Command` can be reused to spwan multiple processes. The builder methods change the command without needing to immediately spwan the process.

```rust
use std::process::Command;

let mut echo_hello = Command::new("sh");
echo_hello.arg("-c")
	.arg("echo hello");
let hello_1 = echo_hello.output().expect("failed to execute process");
let hello_1 = echo_hello.output().expect("failed to execute process");
```

Simalarly, you can builder methods after spwaning a process and then spwan a new process with the modified settings.

```rust
use std::process::Command;

let mut list_dir = Command::new("ls");

// Execute `ls` in the current directory of the program.
list_dir.status().expect("process failed to execute");

println!();

// Change `ls` to execute in the root directory.
list_dir.current_dir("/");

// And then execute `ls` again but in the root directory
list_dir.status().expect("process failed to execute");
```

1. `fn new(program: S) -> Command`

   Constructs a new `Command` for lauching the program at path `program`.

   **Remark:** The new `Command` will inherit the current process's  working directory.

   Builder methods are provided to change the configurations.

   ```rust
   use std::process::Command;
   
   Command::new("sh")
   		.spawn()
   		.expect("sh command failed to start");
   ```

2. `fn arg(&mut self, arg: S) -> &mut Command`

   Adds an argument to pass to the program.

   Only one argument can be passed per use.

   So `.arg("-C /path/to/repo")` is invalid.

   The correct usage would be:

   ```rust
   .arg("-C")
   .arg("/path/to/repo")
   ```

   Note that the argument is not passed through a shell, but given literally to the program. 

   ```rust
   use std::process::Command;
   
   Command::new("ls")
   		.arg("-l")
   		.arg("-a")
   		.spawn()
   		.expect("ls command failed to start");
   ```

3. `fn env(&mut self, key: K, val: V) -> &mut Command`

   Inserts or updates an environment variable mapping.

   Note that environment variable names are case-instensitive, (but case-preserving) on Windows, and case-sensitive on all other platforms.

   ```rust
   use std::process::Command;
   
   Command::new("ls")
   		.env("PATH", "bin")
   		.spwan()
   		.expect("ls command failed to start");
   ```

4. `fn env_remove(&mut self, key: K) -> &mut Command`

   Removes an environment variable mapping.

5. `fn current_dir(&mut self, dir: P) -> &mut Command`

   Sets the working directory for the child process.

   ```rust
   use std::process::Command;
   
   Command::new("ls")
   		.current_dir("/bin")
   		.spawn()
   		.expect("ls command failed to start");
   ```

6. `fn stdin(&mut self, cfg: T) -> &mut Command`

   Configuration for the child process's standard input(stdin) handle.

   ```rust
   use std::process::{Command, Stdio};
   
   Command::new("ls")
   		.stdin(Stdio::null())
   		.spawn()
   		.expect("ls command failed to start");
   ```

7. `fn stdout(&mut self, cfg: T) -> &mut Command`

   Configuration for the child process's standard output (stdout) handle.

8. `fn stderr(&mut self, cfg: T) -> &mut Command`

   Configuration for the child process's standard err (stderr) handle.

9. `fn spawn(&mut self) -> Result<Child>`

   Executes the command as a child process, returning a handle to it. By default, stdin, stdout and stderr are inherited from the parent.

   ```rust
   use std::process::Command;
   
   Command::new("ls")
   		.spqwn()
   		.expect("ls command failed to start");
   ```

10. `fn get_program(&self) -> &OsStr`

    ```rust
    use std::process::Command;
    
    let cmd = Command::new("echo");
    assert_eq!(cmd.get_program(), "echo");
    ```



## std::fs::create_dir

`fn create_dir(path: P) -> Result<()>`

Creates a new, empty directory at the provided path.

**Platform-spefic behavior:**

This function currently corresponds to the `mkdir` function on Unix and the `CreateDirectory` function on Windows.

*Note:* If a parent of the given path doesn't exist, this function will return an error. 

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    fs::create_dir("/some/dir")?;
    Ok(())
}
```

## Arc<Mutex\<T>>

use `*` to modify the value:

```rust
let mut job_num = global::JOB_NUM.lock().unwrap();
*job_num += 1;
```

## std::fs::remove_dir_all

`fn remove_dir_all(path: P) -> Result<()>`

Removes a directory at this path, after removing all its contests.

## std::fs::File



## std::time::Instant

A measurement of a monotonically nodecreasing clock. Opaque and useful only with `Duration`. Note that the instants are **not** guaranted to be **steady**, i.e. some seconds may be longer than others. Instants are opaque types that can only be compared to one another. <u>There is no method to get "the number of seconds" from an instant.</u> Instead, it only allows measuring the duration between two instants (or comparing teo instants).

```rust
use std::time::{Duration, Instant};
use std::thread::sleep;

fn main()
{
    let now = Instant::now();
    
    // we sleep for 2 seconds
    sleep(Duration::new(2, 0));
    // it prints '2'
    println!("{}", now.elapsed.as_secs());
}
```

**OS-spefic behaviors**

An `Instant` is a wrapper around system-specific types and it may behave differently depending on the underlying operation system.

**Implementations:**

1. `fn now() -> Instant`

   Returns an instant corresponding to "now".

2. `fn duration_since(&self, earlirr: Instant) -> Duration`

3. Returns the amount of time elapsed from another to this one, or zero duration if that instant is later than this one.

   ```rust
   use std::time::{Duration, Instant};
   use std::thread::sleep;
   
   let now = Instant::now();
   sleep(Duration::new(1, 0));
   let new_now = Instant::now();
   println!("{:?}", new_now.duration_since(now));
   println!("{:?}", now.duration_since(new_now)); // 0ns
   ```

## Actix-web Extractors

Extractors, i.e. `impl FromRequest`

An extractor can be accessed as an argument to a handler function.

**Argument position does not matter.**

```rust
async fn index(path: Web::Path<(String, String)>, json: web::Json<Myinfo>) -> impl Responder {
    let path = path.into_inner();
    format!("{} {} {} {}", path.0, path.1, json.id, json.username)
}
```

### Path

*Path* peovides information that is extracted from the request's path.

For instance, for resource that registered for the `/users/{user_id}/{friend}` path, `user_id` and `friend` can be deserialized. These segments could be extracted as a tuple in the order they are declared. (e.g. `Path<(u32, String)>`)

> Parts of the path that are extractable are called "dynamic segments" and are marked with curly braces.

```rust
#[get("/users/{user_id}/{friend}")] // <- define path parameters
assync fn index(path: web::Path<(u32, String)>) -> Result<String> {
    let (user_id, friend) = path.into_inner();
    Ok()
}
```

It is also possible to extract path information to a type that implements the Deserialize trait from `serde` by matching dynamic segment names with field names, i.e. uses `serde` instead of a tuple type.

```rust
#[derive(Deserialize)]
struct Info {
    user_id: u32,
    friend: String,
}

// extract path info using serde
#[get("/users/{user_id}/{friend}")] // <- define path parameters
async fn index(info: web::Path<info>) -> Result<String> {
    ...
}
```

### Query

The `Query<T>` type provides extraction functionality for the request's query parameters. Underneath it uses `serde_erlencoded` crate.

```rust
use actix_web::{get, web, App, HttpServer};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    username: String,
}

// this handle gets called if the query deserializes into Info successfully
// otherwise a 400 Bad Request error response is returned
#[get("/")]
async fn index(info: web::Query<Info>) -> String {
    format!("Welcome {}!", info.username)
}
```

## http::request

HTTP request types.

This module contains structs related to HTTP requests, notably the `Request` type itself as well as a builder to create requests. Typically you'll import the `http::Request` type rather than reaching into this module itself.

Creating a `Request` to send:

```rust
use http::{Request, Response};

let mut request = Request::builder()
	.uri("https://www.rust-lang.org")
	.header("User-Agent", "my-awesome-agent/1.0");

if needs_awesome_header() {
    request = request.header("Awesome", "yes");
}

let response = send(request.body()).unwrap();

fn send(req: Request<()>) -> Response<()> {
    // ...
}
```

Inspecting a request to see what was sent:

```rust
use http::{Request, Response, StatusCode};

fn respond_to(req: Request<()>) -> http::Result<Response<()>> {
    if req.uri() != "/awesome_url" {
        return Response::builder()
        	.status(StatusCode::NOT_FOUND)
        	.body(())
    }
    
    let has_awesome_header = req.headers().contains_kry("Awesome");
    let body = req.body();
    // ...
}
```



# Issues

1. How to run the first test case?

   `cargo test -- test_01_15_pts_basic_judging`

2. Can `std::process::Command` realize the complex commands with `|`?

   In Rust, use `Stdio::piped()` to simulate this.

3. Rust-analyzer -- `Failed to run build scripts of some packages, ...`

   Try:

   `sudo apt-get install -y openssl`    

   `sudo apt install pkg-config`     

   `sudo apt install libssl-dev`

4. Automatic test -- can't connect the OJ?

   read the `.stderr` file, parse the `--flush-data`

5. `#![feature(const_fn)]`, feature has been removed.

   use the `Arc` and `Mutex` in std repo.

6. `temporary value dropped while borrowed`

   The String created by `format!()` is temporary, you need to `clone` it and transfer the ownership.

   ```rust
   let s = format!("/TMPDIR_{}", job_num).clone();
   ```

7. If the test is right, the Server will be killed, otherwise it will be remained and you can send requests to debug. Use `killall oj` to kill the course named `oj`.

8. How to end the program when time is up?

   The `Child` type has the method `kill()`

9. 



# Test

## Test Command

The basic automatic test:

```rust
cargo test --test basic_requirements -- --test-threads=1
```

Only test the first case:

```rust
cargo test -- test_01_15_pts_basic_judging
```

Test the third case:

```
cargo test -- test_03_10_pts_job_list
```

Test the fourth case:

```rust
cargo test -- test_04_5_pts_user_support
```

Test the fifth case:

```rust
cargo test -- test_05_5_pts_ranklist_support
```

Using `cargo run`:

```shell
cargo run -- --config tests/cases/01_01_hello_world.config.json --flush-data
```

```rust
cargo run -- --config tests/cases/01_02_wrong_results.config.json --flush-data
```

```rust
cargo run -- --config tests/cases/01_04_time_limit_exceeded.config.json --flush-data
```

```rust
cargo run -- --config tests/cases/03_01_job_list.config.json --flush-data
```

```rust
cargo run -- --config tests/cases/03_02_job_list_with_filter.config.json --flush-data
```

```rust
cargo run -- --config tests/cases/03_03_rejudging.config.json --flush-data
```

```rust
cargo run -- --config tests/cases/05_01_global_ranklist.config.json --flush-data
```



# Advanced Requirements

Test the advanced requirements:

```rust
cargo test --test advanced_requirements -- --test-threads=1
```

## adv_01_contest_support

The test requests:

- post/users user1
- post/contests Contest 1
- post/contests Contest 2
- get/contests
- get/contests/1
- get/contests/3 -> `404 Not Found`
- post/jobs user_id = 1, problem_id = 1, contest_id = 2
- post/jobs user_id = 0, problem_id = 0, contest_id = 2
- post/jobs
- post/jobs
- get/contests/1/ranklist
- get/contests/2/ranklist

```rust
cargo test -- test_adv_01_10_pts_contest_support
```

```rust
cargo run -- --config tests/cases/adv_01_contest_support.config.json --flush-data
```

## adv_02_persistent_storage

```rust
cargo test -- test_adv_02_10_pts_persistent_storage
```

```rust
cargo run -- --config tests/cases/adv_02_persistent_storage.config.json --flush-data
```

## adv_05_packed_judging

```rust
cargo test -- test_adv_05_5_pts_packed_judging
```

```rust
cargo run -- --config tests/cases/adv_05_packed_judging.config.json --flush-data
```

## adv_06_special_judge

```rust
cargo run -- --config tests/cases/adv_06_special_judge.config.json --flush-data
```

## adv_07_dynamic_ranking

```rust
cargo run -- --config tests/cases/adv_07_dynamic_ranking.config.json --flush-data
```

# To Do

judge_task.rs

judge whether problems or users are in the contest

`reason=ERR_INVALID_ARGUMENT, code=1, HTTP 400 Bad Request`