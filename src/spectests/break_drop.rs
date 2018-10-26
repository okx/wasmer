// Rust test file autogenerated with cargo build (src/build_spectests.rs).
// Please do NOT modify it by hand, as it will be reseted on next build.
// Test based on spectests/break_drop.wast
#![allow(
    warnings,
    dead_code
)]
use crate::webassembly::{instantiate, compile, ImportObject, ResultObject, VmCtx, Export};
use super::_common::spectest_importobject;
use wabt::wat2wasm;


// Line 1
fn create_module_1() -> ResultObject {
    let module_str = "(module
      (type (;0;) (func))
      (func (;0;) (type 0)
        block  ;; label = @1
          br 0 (;@1;)
        end)
      (func (;1;) (type 0)
        block  ;; label = @1
          i32.const 1
          br_if 0 (;@1;)
        end)
      (func (;2;) (type 0)
        block  ;; label = @1
          i32.const 0
          br_table 0 (;@1;)
        end)
      (export \"br\" (func 0))
      (export \"br_if\" (func 1))
      (export \"br_table\" (func 2)))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

// Line 7
fn l7_assert_return_invoke(result_object: &ResultObject, vm_context: &VmCtx) {
    println!("Executing function {}", "l7_assert_return_invoke");
    let func_index = match result_object.module.info.exports.get("br") {
        Some(&Export::Function(index)) => index,
        _ => panic!("Function not found"),
    };
    let invoke_fn: fn(&VmCtx) = get_instance_function!(result_object.instance, func_index);
    let result = invoke_fn(&vm_context);
    assert_eq!(result, ());
}

// Line 8
fn l8_assert_return_invoke(result_object: &ResultObject, vm_context: &VmCtx) {
    println!("Executing function {}", "l8_assert_return_invoke");
    let func_index = match result_object.module.info.exports.get("br_if") {
        Some(&Export::Function(index)) => index,
        _ => panic!("Function not found"),
    };
    let invoke_fn: fn(&VmCtx) = get_instance_function!(result_object.instance, func_index);
    let result = invoke_fn(&vm_context);
    assert_eq!(result, ());
}

// Line 9
fn l9_assert_return_invoke(result_object: &ResultObject, vm_context: &VmCtx) {
    println!("Executing function {}", "l9_assert_return_invoke");
    let func_index = match result_object.module.info.exports.get("br_table") {
        Some(&Export::Function(index)) => index,
        _ => panic!("Function not found"),
    };
    let invoke_fn: fn(&VmCtx) = get_instance_function!(result_object.instance, func_index);
    let result = invoke_fn(&vm_context);
    assert_eq!(result, ());
}

#[test]
fn test_module_1() {
    let result_object = create_module_1();
    let vm_context = result_object.instance.generate_context();
    // We group the calls together
    l7_assert_return_invoke(&result_object, &vm_context);
    l8_assert_return_invoke(&result_object, &vm_context);
    l9_assert_return_invoke(&result_object, &vm_context);
}
