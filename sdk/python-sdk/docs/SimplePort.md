# SimplePort

The simple representation of a port

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | Uuid of the port | 
**port** | **int** | Port number | 
**protocol** | [**PortProtocol**](PortProtocol.md) |  | 
**host** | **str** | The host this port is assigned to | 
**comment** | **str** | A comment to the port | 
**workspace** | **str** | The workspace this port is linked to | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.simple_port import SimplePort

# TODO update the JSON string below
json = "{}"
# create an instance of SimplePort from a JSON string
simple_port_instance = SimplePort.from_json(json)
# print the JSON string representation of the object
print SimplePort.to_json()

# convert the object into a dict
simple_port_dict = simple_port_instance.to_dict()
# create an instance of SimplePort from a dict
simple_port_form_dict = simple_port.from_dict(simple_port_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


