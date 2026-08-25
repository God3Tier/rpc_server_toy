FROM python:3.12-slim
WORKDIR /work

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl xz-utils build-essential ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/*

# Adjust to wherever your actual mock/test script and its deps live.
COPY mock/requirements.txt ./requirements.txt
RUN pip install --no-cache-dir -r requirements.txt
COPY ./mock/ctypes_test.py ./ctypes_test.py
COPY mock/mock_test.py ./mock_test.py
ENTRYPOINT ["python", "mock_test.py"]
