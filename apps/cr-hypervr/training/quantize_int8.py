from __future__ import annotations

from pathlib import Path
from onnxruntime.quantization import quantize_dynamic, QuantType


def quantize(onnx_in: str = "models/movie-minilm-v1/model.onnx", onnx_out: str = "models/movie-minilm-v1/model-int8.onnx") -> None:
    Path(Path(onnx_out).parent).mkdir(parents=True, exist_ok=True)
    quantize_dynamic(
        model_input=onnx_in,
        model_output=onnx_out,
        weight_type=QuantType.QInt8,
        optimize_model=True,
    )
    print("Quantized INT8 ONNX saved to", onnx_out)


if __name__ == "__main__":
    quantize()

