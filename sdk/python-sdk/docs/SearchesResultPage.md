# SearchesResultPage

Response containing paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[SearchEntry]**](SearchEntry.md) | The page&#39;s items | 
**limit** | **int** | The limit this page was retrieved with | 
**offset** | **int** | The offset this page was retrieved with | 
**total** | **int** | The total number of items this page is a subset of | 

## Example

```python
from kraken_sdk.models.searches_result_page import SearchesResultPage

# TODO update the JSON string below
json = "{}"
# create an instance of SearchesResultPage from a JSON string
searches_result_page_instance = SearchesResultPage.from_json(json)
# print the JSON string representation of the object
print SearchesResultPage.to_json()

# convert the object into a dict
searches_result_page_dict = searches_result_page_instance.to_dict()
# create an instance of SearchesResultPage from a dict
searches_result_page_form_dict = searches_result_page.from_dict(searches_result_page_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


