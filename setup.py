from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="rusty_automata",
    rust_extensions=[
        RustExtension(
            "rusty_automata._rusty_automata",
            binding=Binding.PyO3,
            # Between our use of editable installs and wanting to use Rust for performance sensitive
            # code, it makes sense to just always use --release
            debug=False,
        )
    ],
    package_data={"rusty_automata": ["py.typed"]},
    packages=["rusty_automata"],
    zip_safe=False,
)
