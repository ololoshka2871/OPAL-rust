### Generate protocol_pb2.py for test

```bash
$ protoc --proto_path=../src/protobuf --python_out=. ../src/protobuf/ProtobufDevice_0000E006.proto
$ mv ProtobufDevice_0000E006_pb2.py protocol_pb2.py
```
