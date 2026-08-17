FROM python:3.12-slim
WORKDIR /work
# Adjust to wherever your actual mock/test script and its deps live.
COPY mock/requirements.txt ./requirements.txt
RUN pip install --no-cache-dir -r requirements.txt
COPY mock/mock_test.py ./mock_test.py
ENTRYPOINT ["python", "mock_test.py"]
