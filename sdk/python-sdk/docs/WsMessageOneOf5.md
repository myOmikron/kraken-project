# WsMessageOneOf5

A notification about a search result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**search_uuid** | **str** | The corresponding id of the search results | 
**result_uuid** | **str** | A result entry | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of5 import WsMessageOneOf5

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf5 from a JSON string
ws_message_one_of5_instance = WsMessageOneOf5.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf5.to_json()

# convert the object into a dict
ws_message_one_of5_dict = ws_message_one_of5_instance.to_dict()
# create an instance of WsMessageOneOf5 from a dict
ws_message_one_of5_form_dict = ws_message_one_of5.from_dict(ws_message_one_of5_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


