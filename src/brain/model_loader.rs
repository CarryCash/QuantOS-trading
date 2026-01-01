use pyo3::prelude::*;
use pyo3::types::PyList;

pub struct QuantosBrain {
    model: PyObject,
}


impl QuantosBrain {
    pub fn new(model_path: &str) -> PyResult<Self> {
        Python::with_gil(|py| {
            // 1. Forzamos la ruta del venv en el sys.path de Python

            

            // Dentro de Python::with_gil(|py| { ...
            let warnings = py.import("warnings")?;
            warnings.call_method1("filterwarnings", ("ignore",))?;
            let sys = py.import("sys")?;
            let path: &PyList = sys.getattr("path")?.downcast()?;
            
            // Reemplaza con tu ruta exacta a site-packages
            let venv_site_packages = r"C:\Users\USER\Downloads\QuantOS-Core\venv\Lib\site-packages";
            path.insert(0, venv_site_packages)?;

            // 2. Ahora intentamos cargar joblib
            let joblib = py.import("joblib")?;
            let model = joblib.call_method1("load", (model_path,))?;
            
            Ok(QuantosBrain {
                model: model.into(),
            })
        })
    }

    pub fn predict_noise(&self, features: Vec<f64>) -> PyResult<f64> {
        Python::with_gil(|py| {
            let model = self.model.as_ref(py);
            let prediction = model.call_method1("predict_proba", (vec![features],))?;
            let proba: Vec<Vec<f64>> = prediction.extract()?;
            Ok(proba[0][1])
        })
    }
}