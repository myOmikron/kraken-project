# UpdateMeRequest

The request to update the own user  At least one of the options must be set

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**username** | **str** |  | [optional] 
**display_name** | **str** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_me_request import UpdateMeRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateMeRequest from a JSON string
update_me_request_instance = UpdateMeRequest.from_json(json)
# print the JSON string representation of the object
print UpdateMeRequest.to_json()

# convert the object into a dict
update_me_request_dict = update_me_request_instance.to_dict()
# create an instance of UpdateMeRequest from a dict
update_me_request_form_dict = update_me_request.from_dict(update_me_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


