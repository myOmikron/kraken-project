# WsMessage

Message that is sent via websocket

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**type** | **str** |  | 
**invitation_uuid** | **str** | The uuid of the invitation | 
**workspace** | [**SimpleWorkspace**](SimpleWorkspace.md) |  | 
**var_from** | [**SimpleUser**](SimpleUser.md) |  | 
**attack** | [**SimpleAttack**](SimpleAttack.md) |  | 
**search_uuid** | **str** | The corresponding id of the search results | 
**finished_successful** | **bool** | Whether the search was finished successfully | 
**result_uuid** | **str** | A result entry | 
**attack_uuid** | **str** | The corresponding id of the attack | 
**source** | **str** | The source address that was queried | 
**destination** | **str** | The destination address that was returned | 
**host** | **str** | A host which could be reached | 
**address** | **str** | The address of the result | 
**port** | **int** | The port of the result | 
**entries** | [**List[CertificateTransparencyEntry]**](CertificateTransparencyEntry.md) | The entries of the result | 
**service** | **str** | Name of the service | 

## Example

```python
from kraken_sdk.models.ws_message import WsMessage

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessage from a JSON string
ws_message_instance = WsMessage.from_json(json)
# print the JSON string representation of the object
print WsMessage.to_json()

# convert the object into a dict
ws_message_dict = ws_message_instance.to_dict()
# create an instance of WsMessage from a dict
ws_message_form_dict = ws_message.from_dict(ws_message_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


