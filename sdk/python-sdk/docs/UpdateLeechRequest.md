# UpdateLeechRequest

The request to update a leech

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | [optional] 
**address** | **str** |  | [optional] 
**description** | **str** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_leech_request import UpdateLeechRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateLeechRequest from a JSON string
update_leech_request_instance = UpdateLeechRequest.from_json(json)
# print the JSON string representation of the object
print UpdateLeechRequest.to_json()

# convert the object into a dict
update_leech_request_dict = update_leech_request_instance.to_dict()
# create an instance of UpdateLeechRequest from a dict
update_leech_request_form_dict = update_leech_request.from_dict(update_leech_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


