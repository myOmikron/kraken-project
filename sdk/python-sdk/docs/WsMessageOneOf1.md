# WsMessageOneOf1

An invitation to a workspace was issued

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**invitation_uuid** | **str** | The uuid of the invitation | 
**workspace** | [**SimpleWorkspace**](SimpleWorkspace.md) |  | 
**var_from** | [**SimpleUser**](SimpleUser.md) |  | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of1 import WsMessageOneOf1

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf1 from a JSON string
ws_message_one_of1_instance = WsMessageOneOf1.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf1.to_json()

# convert the object into a dict
ws_message_one_of1_dict = ws_message_one_of1_instance.to_dict()
# create an instance of WsMessageOneOf1 from a dict
ws_message_one_of1_form_dict = ws_message_one_of1.from_dict(ws_message_one_of1_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


