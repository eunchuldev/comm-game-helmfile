use crate::lib::*;
use lazy_static::lazy_static;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
pub fn encode(text: String) -> PyResult<String> {
    let text = control_chars(text, "_");
    let text = derepeat(text, 5);
    let text = whitespace_less(text);
    let text = hangul_to_jamo(text);
    Ok(text)
}

#[pymodule]
pub fn hangul_normalize(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    Ok(())
}
