# CreateLeechRequest

The request to create a new leech

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | 
**address** | **str** |  | 
**description** | **str** |  | [optional] 

## Example

```python
from kraken_sdk.models.create_leech_request import CreateLeechRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateLeechRequest from a JSON string
create_leech_request_instance = CreateLeechRequest.from_json(json)
# print the JSON string representation of the object
print CreateLeechRequest.to_json()

# convert the object into a dict
create_leech_request_dict = create_leech_request_instance.to_dict()
# create an instance of CreateLeechRequest from a dict
create_leech_request_form_dict = create_leech_request.from_dict(create_leech_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


