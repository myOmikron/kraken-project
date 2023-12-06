# ServiceResultsPage

Response containing paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[FullService]**](FullService.md) | The page&#39;s items | 
**limit** | **int** | The limit this page was retrieved with | 
**offset** | **int** | The offset this page was retrieved with | 
**total** | **int** | The total number of items this page is a subset of | 

## Example

```python
from kraken_sdk.models.service_results_page import ServiceResultsPage

# TODO update the JSON string below
json = "{}"
# create an instance of ServiceResultsPage from a JSON string
service_results_page_instance = ServiceResultsPage.from_json(json)
# print the JSON string representation of the object
print ServiceResultsPage.to_json()

# convert the object into a dict
service_results_page_dict = service_results_page_instance.to_dict()
# create an instance of ServiceResultsPage from a dict
service_results_page_form_dict = service_results_page.from_dict(service_results_page_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)

