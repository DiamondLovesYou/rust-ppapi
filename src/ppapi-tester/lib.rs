
// Provides the ```pnacl_test! {}``` macro.
// Usage:
//
// try one:
// ```rust
// pnacl_test! {
//    fn test_name() {
//        extern crate ppapi;
//        use std::collections::HashMap;
//        #[no_mangle]
//        pub extern fn ppapi_instance_created(_instance: ppapi::Instance,
//                                             _args: HashMap<String, String>) {
//            // Test logic here.
//        }
//    }
// }
// ```
// try two:
// ```rust
// #[ppapi_test]
// fn test_name(instance: ppapi::Instance) {
// }
// ```

#![crate_name = "ppapi-tester"]
#![crate_type = "dylib"]
#![feature(plugin_registrar, quote)]

#![allow(unstable)]

extern crate rustc;
extern crate syntax;

use syntax::ext::base::{SyntaxExtension, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::quote::rt::{ToSource, ToTokens};
use syntax::parse::token::{intern, intern_and_get_ident};
use syntax::{ast, codemap};
use syntax::codemap::Span;
use syntax::ptr::P;
use rustc::plugin::Registry;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let ppapi_test_expander = SyntaxExtension::Modifier(Box::new(expand_test));
    let ppapi_test = intern("ppapi_test");
    reg.register_syntax_extension(ppapi_test, ppapi_test_expander);
}

const HTML: &'static str = include_str!("index.html");
const MANIFEST: &'static str = include_str!("manifest.nmf");

fn expand_test(ecx: &mut ExtCtxt,
               sp: Span,
               _meta_item: &ast::MetaItem,
               item: P<ast::Item>) -> P<ast::Item> {
    // we replace the input function with a function that instead compiles the
    // pretty printed original to a pexe and invokes chrome to simulate being
    // embedding in a website.

    println!("============");
    println!("{:?}", item);

    #[derive(PartialEq)]
    enum HasTestSignature {
        Yes,
        No,
        NotEvenAFunction,
    }

    fn has_correct_signature(i: &ast::Item) -> HasTestSignature {
        match &i.node {
          &ast::ItemFn(ref decl, _, _, ref generics, _) => {
            let no_output = match decl.output {
                ast::DefaultReturn(..) => true,
                _ => false
            };
            if decl.inputs.len() == 2
                   && no_output
                   && !generics.is_parameterized() {
                HasTestSignature::Yes
            } else {
                HasTestSignature::No
            }
          }
          _ => HasTestSignature::NotEvenAFunction,
        }
    }

    // first check that the annotated item is, in fact, a function:
    match has_correct_signature(&*item) {
        HasTestSignature::Yes => { println!("sig is fine"); },
        HasTestSignature::No => {
            ecx.span_err(sp, "function must be typed: fn(instance: \
                         ppapi::Instance, args: HashMap<String, String>) -> ()");
            return item;
        },
        HasTestSignature::NotEvenAFunction => {
            ecx.span_err(sp, "must be a function: fn(instance: ppapi::Instance, \
                         args: HashMap<String, String>) -> ()");
            return item;
        },
    }

    let test_name = item.ident;
    let wrapper =
        quote_item!(
            ecx,
            #![no_main]
            extern crate ppapi;

            use std::collections::HashMap;

            struct ResultMessager(ppapi::Messaging);
            impl Drop for ResultMessager {
                fn drop(&mut self) {
                    use std::rt::unwind::panicking;
                    let &mut ResultMessager(msg) = self;

                    if panicking() {
                        msg.post_message("failure");
                    } else {
                        msg.post_message("success");
                    }
                }
            }

            $item

            #[no_mangle] pub extern fn ppapi_instance_created(instance: ppapi::Instance,
                                                              args: HashMap<String, String>) {
                use ppapi::MessageLoop;
                assert!(MessageLoop::post_to_self(move |: _| {
                    let result = ResultMessager(instance.messaging());
                    $test_name(instance.clone(), args.clone());
                }).is_ok());
                assert!(MessageLoop::current().unwrap().shutdown().is_ok());
            }
            #[no_mangle] pub extern fn ppapi_instance_destroyed() { }
            );
    let wrapper = wrapper.expect("the wrapper failed to quote");
    let wrapper_ident = intern_and_get_ident(wrapper.to_source().as_slice());
    let wrapper_str = ecx.expr_str(sp, wrapper_ident);

    let cwd = ::std::os::getcwd().unwrap();
    let deps_str = cwd.join_many(["target", "le32-unknown-nacl", "deps"].as_slice())
        .display().to_string();
    let deps_ident = intern_and_get_ident(deps_str.as_slice());
    let deps_dir = ecx.expr_str(codemap::DUMMY_SP, deps_ident);

    let timeout = None::<u64>;

    let host_test =
        quote_item!(
            ecx,
            #[test] fn $test_name() {
                use std::io::TempDir;
                use std::io::fs::File;
                use std::io::{Command, LineBufferedWriter};
                use std::io::process::{ProcessExit, ProcessOutput};
                use std::io::stdio::{stdout, stderr, StdWriter};

                // the source to the real test:
                const REAL_TEST: &'static str = $wrapper_str;

                let tmp = TempDir::new(stringify!($test_name))
                    .ok().expect("need temp directory for test artifacts");

                let url = format!("file://{}",
                                  tmp.path().join("index.html").display());
                let chrome_args = ["--allow-file-access-from-files".to_string(),
                                   "--bwsi".to_string(),
                                   "--no-sandbox".to_string(),
                                   "--silent-launch".to_string(),
                                   "--noerrdialogs".to_string(),
                                   "--no-first-run".to_string(),
                                   "--no-default-browser-check".to_string(),
                                   url];

                let stdout_path = tmp.path().join("stdout");
                let stdout_name = stdout_path
                    .display().to_string();
                let stderr_path = tmp.path().join("stderr");
                let stderr_name = stderr_path
                    .display().to_string();
                let chrome_env = [("NACL_EXE_STDOUT".to_string(), stdout_name),
                                  ("NACL_EXE_STDERR".to_string(), stderr_name)];

                let rustc_args = ["-".to_string(),
                                  "-o".to_string(),
                                  tmp.path().join("test-main.pexe").display().to_string(),
                                  "--crate-type".to_string(), "bin".to_string(),
                                  "--target".to_string(),
                                  "le32-unknown-nacl".to_string(),
                                  "-L".to_string(),
                                  ($deps_dir).to_string()];
                let mut rustc = Command::new("rustc");
                rustc.args(rustc_args);
                println!("∨∨∨∨∨∨∨∨∨∨∨    rustc    ∨∨∨∨∨∨∨∨∨∨∨");
                println!("spawning `{:?}`", rustc);
                let mut rustc = rustc.spawn().unwrap();
                rustc.stdin
                    .as_mut()
                    .unwrap()
                    .write_str(REAL_TEST)
                    .unwrap();

                {
                    let path = tmp.path()
                        .join("index.html");
                    let file = File::create(&path).unwrap();
                    file.write_str($HTML).unwrap();
                }
                {
                    let path = tmp.path()
                        .join("manifest.nmf");
                    let file = File::create(&path).unwrap();
                    file.write_str($MANIFEST).unwrap();
                }

                match rustc.wait_with_output() {
                    Ok(ProcessOutput {
                        status: exit_status,
                        output: out, error: err,
                    }) => {
                        stdout().write(out.as_slice());
                        stderr().write(err.as_slice());
                        assert!(exit_status.success());
                    }
                    err => { err.unwrap(); },
                }
                println!("∧∧∧∧∧∧∧∧∧∧∧    rustc    ∧∧∧∧∧∧∧∧∧∧∧");

                let mut chrome = Command::new("google-chrome");
                chrome.args(chrome_args);
                for (k, v) in chrome_env.iter() {
                    chrome.env(k, v);
                }
                println!("∨∨∨∨∨∨∨∨∨∨∨   chrome    ∨∨∨∨∨∨∨∨∨∨∨");
                println!("spawning `{:?}`:", chrome);

                let mut chrome = chrome.spawn().unwrap();
                chrome.set_timeout($timeout);

                fn split_and_print(where_: LineBufferedWriter<StdWriter>,
                                   kind: &'static str, out: &[u8]) {
                    for line in out.split(move |: v| v == '\n' ) {
                        where_.write_str(kind).unwrap();
                        where_.write(line).unwrap();
                        where_.write_u8('\n').unwrap();
                    }
                }

                fn dump_output(cmd: &mut Process) {
                    let output = cmd.stdout
                        .as_mut().unwrap()
                        .read_to_end().unwrap();
                    let error = cmd.stderr
                        .as_mut().unwrap()
                        .read_to_end().unwrap();
                    split_and_print(stdout(), "stdout: ", output.as_slice());
                    split_and_print(stderr(), "stderr: ", error.as_slice());
                }

                match chrome.wait() {
                    Ok(status) => {
                        dump_output(&mut chrome);
                        assert!(status.success());
                    },
                    Err(IoError { kind: IoErrorKind::TimedOut, .. }) => {
                        println!("timeout! sending sigterm...");
                        chrome.set_timeout(Some(6000));
                        chrome.signal_exit().unwrap();
                        let res = chrome.wait();
                        let res = match res {
                            Err(IoError { kind: IoErrorKind::TimedOut, .. }) => {
                                println!("1s later: chrome still hasn't quit; killing.");
                                chrome.signal_kill().unwrap();
                                chrome.wait()
                            },
                            res => res,
                        }
                        dump_output(&mut chrome);
                        assert_eq!(res, Ok(ProcessExit::ExitStatus(0))); // always false.
                        unreachable!()
                    },
                    res => {
                        dump_output(&mut chrome);
                        res.unwrap();
                        unreachable!()
                    }
                }
                println!("∧∧∧∧∧∧∧∧∧∧∧   chrome    ∧∧∧∧∧∧∧∧∧∧∧");

                println!("∨∨∨∨∨∨∨∨∨∨∨ test output ∨∨∨∨∨∨∨∨∨∨∨");
                fn read_log(p: &Path) -> Vec<u8> {
                    let f = File::open(p).unwrap();
                    f.read_to_end().unwrap()
                }
                let test_stdout = read_log(&stdout_path);
                let test_stderr = read_log(&stderr_path);
                split_and_print(stdout(), "test stdout: ", test_stdout.as_slice());
                split_and_print(stderr(), "test stderr: ", test_stderr.as_slice());
                println!("∧∧∧∧∧∧∧∧∧∧∧ test output ∧∧∧∧∧∧∧∧∧∧∧");
            });
    host_test.unwrap()
}
