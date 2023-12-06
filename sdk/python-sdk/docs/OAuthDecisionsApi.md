# kraken_sdk.OAuthDecisionsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_decisions**](OAuthDecisionsApi.md#get_decisions) | **GET** /api/v1/oauthDecisions | Retrieve a user&#39;s remembered oauth decisions
[**revoke_decision**](OAuthDecisionsApi.md#revoke_decision) | **DELETE** /api/v1/oauthDecisions/{uuid} | Revoke a user&#39;s remembered oauth decision


# **get_decisions**
> GetMyDecisionsResponse get_decisions()

Retrieve a user's remembered oauth decisions

Retrieve a user's remembered oauth decisions

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_my_decisions_response import GetMyDecisionsResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.OAuthDecisionsApi(api_client)

    try:
        # Retrieve a user's remembered oauth decisions
        api_response = await api_instance.get_decisions()
        print("The response of OAuthDecisionsApi->get_decisions:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling OAuthDecisionsApi->get_decisions: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetMyDecisionsResponse**](GetMyDecisionsResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The user&#39;s remember oauth decisions |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **revoke_decision**
> revoke_decision(uuid)

Revoke a user's remembered oauth decision

Revoke a user's remembered oauth decision

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.OAuthDecisionsApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Revoke a user's remembered oauth decision
        await api_instance.revoke_decision(uuid)
    except Exception as e:
        print("Exception when calling OAuthDecisionsApi->revoke_decision: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Revoked decision |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

