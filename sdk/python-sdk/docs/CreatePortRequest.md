# CreatePortRequest

The request to manually add a port

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ip_addr** | **str** | The ip address the port is open on | 
**port** | **int** | The port to add | 
**certainty** | [**ManualPortCertainty**](ManualPortCertainty.md) |  | 
**protocol** | [**PortProtocol**](PortProtocol.md) |  | 

## Example

```python
from kraken_sdk.models.create_port_request import CreatePortRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreatePortRequest from a JSON string
create_port_request_instance = CreatePortRequest.from_json(json)
# print the JSON string representation of the object
print CreatePortRequest.to_json()

# convert the object into a dict
create_port_request_dict = create_port_request_instance.to_dict()
# create an instance of CreatePortRequest from a dict
create_port_request_form_dict = create_port_request.from_dict(create_port_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


