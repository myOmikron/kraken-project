# GetMyDecisionsResponse

Response holding a user's oauth decisions

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**decisions** | [**List[FullDecision]**](FullDecision.md) | A user&#39;s oauth decisions | 

## Example

```python
from kraken_sdk.models.get_my_decisions_response import GetMyDecisionsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetMyDecisionsResponse from a JSON string
get_my_decisions_response_instance = GetMyDecisionsResponse.from_json(json)
# print the JSON string representation of the object
print GetMyDecisionsResponse.to_json()

# convert the object into a dict
get_my_decisions_response_dict = get_my_decisions_response_instance.to_dict()
# create an instance of GetMyDecisionsResponse from a dict
get_my_decisions_response_form_dict = get_my_decisions_response.from_dict(get_my_decisions_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


