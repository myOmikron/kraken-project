# PageParams

Query parameters for paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**limit** | **int** | Number of items to retrieve | 
**offset** | **int** | Position in the whole list to start retrieving from | 

## Example

```python
from kraken_sdk.models.page_params import PageParams

# TODO update the JSON string below
json = "{}"
# create an instance of PageParams from a JSON string
page_params_instance = PageParams.from_json(json)
# print the JSON string representation of the object
print PageParams.to_json()

# convert the object into a dict
page_params_dict = page_params_instance.to_dict()
# create an instance of PageParams from a dict
page_params_form_dict = page_params.from_dict(page_params_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


