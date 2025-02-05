use pyo3::prelude::*;
#[path = "../../src/file_conc.rs"]
mod file_conc;
use file_conc::concatenate_files as rust_concatenate_files;

#[pyfunction]
fn concatenate_files_py(
    repo_dir: &str,
    ignore: &str,
    target: &str,
    use_clipboard: bool,
) -> PyResult<()> {
    match rust_concatenate_files(&repo_dir, &ignore, &target, use_clipboard) {
        Ok(_) => Ok(()),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            e.to_string(),
        )),
    }
}

#[pymodule]
#[pyo3(name = "reposyn")]
fn py_reposyn(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(concatenate_files_py, m)?)?;
    Ok(())
}
