from setuptools import setup
from setuptools_rust import RustExtension


setup(
    name="hangul-normalize",
    version="0.1.0",
    classifiers=[
        "License :: OSI Approved :: MIT License",
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Programming Language :: Python",
        "Programming Language :: Rust",
        "Operating System :: POSIX",
        "Operating System :: MacOS :: MacOS X",
    ],
    packages=["hangul_normalize"],
    rust_extensions=[RustExtension("hangul_normalize.hangul_normalize", "Cargo.toml", debug=False)],
    include_package_data=True,
    zip_safe=False,
)
