use vice::matrix;
use vice::value::Value;

#[test]
fn test_matrix_creation_and_shape() {
    let zeros = matrix::zeros(3, 4).expect("matrix zeros should succeed");
    let shape = matrix::shape(&zeros).expect("shape should succeed");
    assert_eq!(shape, Value::Array(vec![Value::Int(3), Value::Int(4)]));

    let (r, c, data) = matrix::extract_matrix(&zeros).expect("extract should succeed");
    assert_eq!(r, 3);
    assert_eq!(c, 4);
    assert_eq!(data.len(), 12);
    assert!(data.iter().all(|&v| v == 0.0));
}

#[test]
fn test_matrix_from_array_and_transpose() {
    let arr = vec![
        Value::Float(1.0),
        Value::Float(2.0),
        Value::Float(3.0),
        Value::Float(4.0),
        Value::Float(5.0),
        Value::Float(6.0),
    ];
    let mat = matrix::from_array(&arr, 2, 3).expect("from_array should succeed");
    let trans = matrix::transpose(&mat).expect("transpose should succeed");
    let shape = matrix::shape(&trans).expect("shape should succeed");
    assert_eq!(shape, Value::Array(vec![Value::Int(3), Value::Int(2)]));

    let (_, _, data) = matrix::extract_matrix(&trans).expect("extract trans");
    // Original 2x3:
    // [1, 2, 3]
    // [4, 5, 6]
    // Transposed 3x2:
    // [1, 4]
    // [2, 5]
    // [3, 6]
    assert_eq!(data, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn test_matrix_multiplication() {
    // A: 2x2 [[1, 2], [3, 4]]
    let a_arr = vec![
        Value::Float(1.0),
        Value::Float(2.0),
        Value::Float(3.0),
        Value::Float(4.0),
    ];
    let a = matrix::from_array(&a_arr, 2, 2).expect("matrix a");

    // B: 2x2 [[2, 0], [1, 2]]
    let b_arr = vec![
        Value::Float(2.0),
        Value::Float(0.0),
        Value::Float(1.0),
        Value::Float(2.0),
    ];
    let b = matrix::from_array(&b_arr, 2, 2).expect("matrix b");

    // C = A * B = [[1*2 + 2*1, 1*0 + 2*2], [3*2 + 4*1, 3*0 + 4*2]] = [[4, 4], [10, 8]]
    let c = matrix::mul(&a, &b).expect("matrix mul");
    let (_, _, data) = matrix::extract_matrix(&c).expect("extract c");
    assert_eq!(data, vec![4.0, 4.0, 10.0, 8.0]);
}

#[test]
fn test_matrix_vector_dot_product() {
    let v1 = vec![Value::Float(1.0), Value::Float(2.0), Value::Float(3.0)];
    let v2 = vec![Value::Float(4.0), Value::Float(5.0), Value::Float(6.0)];
    let res = matrix::dot(&v1, &v2).expect("dot product");
    // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    assert_eq!(res, 32.0);
}

#[test]
#[cfg(target_os = "windows")]
fn test_ffi_windows_kernel32() {
    let handle_val = vice::ffi::load("kernel32.dll").expect("load kernel32.dll");
    let pid_val = vice::ffi::call(&handle_val, "GetCurrentProcessId", &[], "i32")
        .expect("call GetCurrentProcessId");
    
    match pid_val {
        Value::Int(pid) => {
            let actual_pid = std::process::id() as i64;
            assert_eq!(pid, actual_pid);
        }
        _ => panic!("Expected Int PID from GetCurrentProcessId"),
    }

    let free_res = vice::ffi::free(&handle_val).expect("free handle");
    assert_eq!(free_res, Value::Bool(true));

    // Calling freed handle should fail
    let err = vice::ffi::call(&handle_val, "GetCurrentProcessId", &[], "i32");
    assert!(err.is_err());
}

#[test]
fn test_http_sandbox_enforcement() {
    let temp_dir = std::env::temp_dir();
    vice::stdlib::set_sandbox_root(Some(temp_dir)).expect("set sandbox root");

    let get_res = vice::net_http::get("https://example.com", None);
    assert!(get_res.is_err());
    assert!(get_res.unwrap_err().contains("sandbox"));

    // Reset sandbox
    let _ = vice::stdlib::set_sandbox_root(None);
}

#[test]
fn test_crypto_sha256_and_base64() {
    let sha = vice::crypto::sha256_hex("hello world");
    assert_eq!(
        sha,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );

    let original = "Vice Language for AI and Modern Developers 2026!";
    let b64 = vice::crypto::base64_encode_str(original);
    let decoded = vice::crypto::base64_decode_str(&b64).expect("decode base64");
    assert_eq!(decoded, original);

    let bytes = vice::crypto::secure_random_bytes(16).expect("random bytes");
    assert_eq!(bytes.len(), 16);
}

#[test]
fn test_regex_operations() {
    assert!(vice::reg_engine::is_match(r"^\d{3}-\d{4}$", "123-4567").unwrap());
    assert!(!vice::reg_engine::is_match(r"^\d{3}-\d{4}$", "123-abcd").unwrap());

    let found = vice::reg_engine::find(r"[a-z]+", "123 apple 456").unwrap();
    assert_eq!(found, Some("apple".to_string()));

    let all = vice::reg_engine::find_all(r"\d+", "id: 42, score: 99, level: 3").unwrap();
    assert_eq!(
        all,
        vec![
            Value::String("42".into()),
            Value::String("99".into()),
            Value::String("3".into())
        ]
    );

    let replaced =
        vice::reg_engine::replace(r"\s+", "hello    world  wide", " ").unwrap();
    assert_eq!(replaced, "hello world wide");

    let parts = vice::reg_engine::split(r",\s*", "a, b,  c,d").unwrap();
    assert_eq!(
        parts,
        vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("d".into())
        ]
    );
}

#[test]
fn test_sqlite_in_memory_crud() {
    let handle = vice::db_sqlite::open(":memory:").expect("open in-memory db");

    let create_res = vice::db_sqlite::execute(
        &handle,
        "CREATE TABLE heroes (id INTEGER PRIMARY KEY, name TEXT, level INTEGER, power REAL)",
        &[],
    )
    .expect("create table");
    assert_eq!(create_res, Value::Int(0));

    let ins1 = vice::db_sqlite::execute(
        &handle,
        "INSERT INTO heroes (name, level, power) VALUES (?, ?, ?)",
        &[
            Value::String("ViceKnight".into()),
            Value::Int(50),
            Value::Float(999.5),
        ],
    )
    .expect("insert hero 1");
    assert_eq!(ins1, Value::Int(1));

    let ins2 = vice::db_sqlite::execute(
        &handle,
        "INSERT INTO heroes (name, level, power) VALUES (?, ?, ?)",
        &[
            Value::String("MageRust".into()),
            Value::Int(45),
            Value::Float(850.0),
        ],
    )
    .expect("insert hero 2");
    assert_eq!(ins2, Value::Int(1));

    let rows_val = vice::db_sqlite::query(
        &handle,
        "SELECT name, level, power FROM heroes ORDER BY level DESC",
        &[],
    )
    .expect("query heroes");

    match rows_val {
        Value::Array(rows) => {
            assert_eq!(rows.len(), 2);
            match &rows[0] {
                Value::Map(m) => {
                    assert_eq!(
                        m.get(&vice::value::MapKey::String("name".into())),
                        Some(&Value::String("ViceKnight".into()))
                    );
                    assert_eq!(
                        m.get(&vice::value::MapKey::String("level".into())),
                        Some(&Value::Int(50))
                    );
                }
                _ => panic!("Expected row map"),
            }
        }
        _ => panic!("Expected array of rows"),
    }

    let closed = vice::db_sqlite::close(&handle).expect("close db");
    assert_eq!(closed, Value::Bool(true));
}

#[test]
fn test_pkg_manager_add_and_list() {
    let temp = std::env::temp_dir().join(format!("vice_pkg_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).expect("create temp dir");

    let manifest = temp.join("vice.toml");
    std::fs::write(
        &manifest,
        "[package]\nname = \"demo-app\"\nversion = \"0.2.0\"\n",
    )
    .expect("write manifest");

    let external_mod = temp.join("calculator.vice");
    std::fs::write(&external_mod, "fn add(a: int, b: int) -> int { return a + b; }\n")
        .expect("write external module");

    let add_res =
        vice::pkg::add_dependency(&temp, external_mod.to_str().unwrap()).expect("pkg add");
    assert!(add_res.contains("successfully added"));

    let list_res = vice::pkg::list_dependencies(&temp).expect("pkg list");
    assert!(list_res.contains("calculator"));

    let lock = temp.join("vice.lock");
    assert!(lock.is_file());

    let copied_mod = temp.join("modules").join("calculator.vice");
    assert!(copied_mod.is_file());

    let _ = std::fs::remove_dir_all(&temp);
}
