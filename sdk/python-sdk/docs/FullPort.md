# FullPort

The full representation of a port

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | Uuid of the port | 
**port** | **int** | Port number | 
**protocol** | [**PortProtocol**](PortProtocol.md) |  | 
**host** | [**SimpleHost**](SimpleHost.md) |  | 
**comment** | **str** | A comment to the port | 
**tags** | [**List[SimpleTag]**](SimpleTag.md) | The tags this port is linked to | 
**workspace** | **str** | The workspace this port is linked to | 
**sources** | [**SimpleAggregationSource**](SimpleAggregationSource.md) |  | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.full_port import FullPort

# TODO update the JSON string below
json = "{}"
# create an instance of FullPort from a JSON string
full_port_instance = FullPort.from_json(json)
# print the JSON string representation of the object
print FullPort.to_json()

# convert the object into a dict
full_port_dict = full_port_instance.to_dict()
# create an instance of FullPort from a dict
full_port_form_dict = full_port.from_dict(full_port_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


