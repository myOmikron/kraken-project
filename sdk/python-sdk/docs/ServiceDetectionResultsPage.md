# ServiceDetectionResultsPage

Response containing paginated data

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**items** | [**List[FullServiceDetectionResult]**](FullServiceDetectionResult.md) | The page&#39;s items | 
**limit** | **int** | The limit this page was retrieved with | 
**offset** | **int** | The offset this page was retrieved with | 
**total** | **int** | The total number of items this page is a subset of | 

## Example

```python
from kraken_sdk.models.service_detection_results_page import ServiceDetectionResultsPage

# TODO update the JSON string below
json = "{}"
# create an instance of ServiceDetectionResultsPage from a JSON string
service_detection_results_page_instance = ServiceDetectionResultsPage.from_json(json)
# print the JSON string representation of the object
print ServiceDetectionResultsPage.to_json()

# convert the object into a dict
service_detection_results_page_dict = service_detection_results_page_instance.to_dict()
# create an instance of ServiceDetectionResultsPage from a dict
service_detection_results_page_form_dict = service_detection_results_page.from_dict(service_detection_results_page_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


