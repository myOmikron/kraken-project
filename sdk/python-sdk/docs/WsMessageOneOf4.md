# WsMessageOneOf4

A notification about a finished search

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**search_uuid** | **str** | The corresponding id of the search | 
**finished_successful** | **bool** | Whether the search was finished successfully | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of4 import WsMessageOneOf4

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf4 from a JSON string
ws_message_one_of4_instance = WsMessageOneOf4.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf4.to_json()

# convert the object into a dict
ws_message_one_of4_dict = ws_message_one_of4_instance.to_dict()
# create an instance of WsMessageOneOf4 from a dict
ws_message_one_of4_form_dict = ws_message_one_of4.from_dict(ws_message_one_of4_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


