
use error_stack::{IntoReport, ResultExt};
use masking::PeekInterface;
use serde::{Deserialize, Serialize};
use crate::{core::errors,types::{self,api, storage::enums}};
use serde_repr::{Serialize_repr, Deserialize_repr};
//TODO: Fill the struct with respective fields
// specifying only required fields
#[derive(Default, Debug, Serialize, Eq, PartialEq)]
pub struct TrustpayPaymentsRequest {
    pub amount: i64,
    pub currency: enums::Currency,
    pub pan: String, // card number
    pub cvv: i64,
    pub exp: String,
    pub redirectUrl: String,
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for TrustpayPaymentsRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsAuthorizeRouterData) -> Result<Self,Self::Error> {
        let auth_type = TrustpayAuthType::try_from(&item.connector_auth_type)?;
        let amount = item.request.amount;
        let currency = item.request.currency;
        let payment_method = match item.request.payment_method_data.clone() {
            api::PaymentMethod::Card(ccard) => {
                ccard
            },
            _ => Err(errors::ConnectorError::NotImplemented(
                "Unknown payment method".to_string(),
            ))?,
        };
        let exp = format!("{}/{}",payment_method.card_exp_month.peek(),payment_method.card_exp_year.peek());
        println!("{}", exp);
        let redirectUrl = item.return_url.clone().unwrap_or("https://test-tpgw.trustpay.eu".to_string());
        let req = Self {
            amount,
            pan: payment_method.card_number.peek().to_string(),
            exp,
            currency,
            redirectUrl,
            cvv: payment_method.card_cvc.peek().parse().into_report().change_context(errors::ConnectorError::RequestEncodingFailed)?,
        };
        println!("requestself{:?}",req);
        Ok(req)

    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct TrustpayAuthType {
    pub(super)    api_key: String
}

impl TryFrom<&types::ConnectorAuthType> for TrustpayAuthType  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        if let types::ConnectorAuthType::HeaderKey { api_key } = _auth_type {
            Ok(Self {
                api_key: api_key.to_string(),
            })
        } else {
            Err(errors::ConnectorError::FailedToObtainAuthType.into())
        }
    }
}

// PaymentsResponse
//TODO: Append the remaining status flags
// #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
// #[serde(rename_all = "lowercase")]
// pub enum TrustpayPaymentStatus {
//     Succeeded,
//     Failed,
//     #[default]
//     Processing,
// }

#[derive(Debug, Serialize_repr, Deserialize_repr , PartialEq, Eq, Clone)]
#[repr(i32)]
pub enum TrustpayPaymentStatus {
    Success=0,
    Pending=1,
    Expired= (-1),
    Error=-2,
    ServerCallFailed=-3,
    AbortedByUser=-4,
    Failure=-255,
}

// 1	Unspecified error
// 4	Validation failed
// 5	Validation failed
// 6	Invalid InstanceId format
// 7	Invalid InstanceId value
// 8	Invalid PAN (unexpected characters)
// 9	Invalid PAN (length)
// 10	Invalid PAN (checksum)
// 11	Invalid CVV format
// 12	Invalid CVV value
// 13	Invalid Card expiration format
// 14	Invalid Card expiration (month)
// 15	Invalid Card expiration (year)
// 16	Invalid Amount format
// 17	Invalid Amount value
// 18	Invalid Currency format
// 19	Invalid Currency value
// 21	Access denied for Instance
// 22	Instance already processed
// 23	Instance already processed
// 25	Failed to create Instance
// 26	Invalid CardId
// 30	Access denied
// 31	Authentication failed
// 32	Invalid API KEY
// 34	Invalid parameter value
// 35	Invalid URL
// 37	Invalid request type
// 39	Invalid parameter length
// 40	Invalid secret
// 41	Invalid secret


impl Default for TrustpayPaymentStatus {
    fn default() -> Self {
        TrustpayPaymentStatus::Failure
    }
}

// 0	Success
// 1	Pending
// -1	Expired
// -2	Error
// -3	Server call failed
// -4	Aborted by user
// -255	Failure


// impl From<TrustpayPaymentStatus> for enums::AttemptStatus {
//     fn from(item: TrustpayPaymentStatus) -> Self {
//         match item {
//             TrustpayPaymentStatus::Succeeded => Self::Charged,
//             TrustpayPaymentStatus::Failed => Self::Failure,
//             TrustpayPaymentStatus::Processing => Self::Authorizing,
//         }
//     }
// }

impl From<TrustpayPaymentStatus> for enums::AttemptStatus {
    fn from(item: TrustpayPaymentStatus) -> Self {
        match item {
            // TODO :: we are adding this
            TrustpayPaymentStatus::Success => Self::Charged,
            TrustpayPaymentStatus::Pending => Self::Pending,
            TrustpayPaymentStatus::Error => Self::Failure,
            TrustpayPaymentStatus::Failure => Self::Failure,
            TrustpayPaymentStatus::ServerCallFailed => Self::Failure,
            TrustpayPaymentStatus::Expired => Self::AuthorizationFailed,
            TrustpayPaymentStatus::AbortedByUser => Self::AuthorizationFailed,

        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustpayPaymentsResponse {
    pub status: TrustpayPaymentStatus,
    pub description: String,
    pub instanceId: String,
    pub paymentStatus: String,
    pub paymentDescription: String,
}

impl<F,T> TryFrom<types::ResponseRouterData<F, TrustpayPaymentsResponse, T, types::PaymentsResponseData>> for types::RouterData<F, T, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: types::ResponseRouterData<F, TrustpayPaymentsResponse, T, types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.status),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.instanceId),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct TrustpayRefundRequest {
    amount : i64,
    instanceId : String,
    currency : enums::Currency,
}

impl<F> TryFrom<&types::RefundsRouterData<F>> for TrustpayRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: &types::RefundsRouterData<F>) -> Result<Self,Self::Error> {
       let auth_type = TrustpayAuthType::try_from(&item.connector_auth_type);
        let amount = item.request.amount;
        let currency = item.request.currency;
        let instanceId = item.request.connector_transaction_id.clone();
        let req = Self {
            amount,
            currency,
            instanceId,
        };
        println!("requestself{:?}",req);
        Ok(req)
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]

pub enum RefundStatus {
    Success=0,
    Pending=1,
    #[default]
    Failure
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Success => Self::Success,
            RefundStatus::Pending => Self::Pending,
            RefundStatus::Failure => Self::Failure
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    status : RefundStatus,
    description: Option<String>,
    instanceId : Option<String>,
    paymentStatus :Option <String>,
    paymentDescription : Option<String>,
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.instanceId.ok_or(errors::ConnectorError::MissingRequiredField { field_name: "instanceId" })?,
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
     type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
         todo!()
     }
 }

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrustpayErrorResponse {
    pub status: i32,
    pub description : String,
    pub errors: Vec<ErrorType>
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorType {
    pub code : i32,
    pub description : String,
}




#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrustPaySyncResponse {
    status : TrustpayPaymentStatus,
    instanceId : String,
    created : String,
    amount : String,
    currency : String,
    reference : Option<String>,
    paymentStatus :Option <String>,
    paymentDescription : Option<String>,
    paymentStatusDetails : Option<PaymentStatusD>,
    threeDSecure : Option<ThreeDS>,
    card : Option<Card> ,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrustPayGenericResponse {
    status : TrustpayPaymentStatus,
    description: Option<String>,
    instanceId : Option<String>,
    redirectUrl : Option<String>,
    redirectParams: Option<String>,
    preconditions: Option<String>,
    paymentStatus :Option <String>,
    paymentDescription : Option<String>,
    paymentStatusDetails : Option<PaymentStatusD>,
}



impl<F, T>
    TryFrom<types::ResponseRouterData<F, TrustPaySyncResponse, T, types::PaymentsResponseData>>
    for types::RouterData<F, T, types::PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::ResponseRouterData<
            F,
            TrustPaySyncResponse,
            T,
            types::PaymentsResponseData,
        >,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(item.response.status.clone()),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.instanceId.clone()),
                redirect: false,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaymentStatusD {
    extendedDescription : String,
    schemeResponseCode : String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThreeDS {
    eci : String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Card {
    maskedPan : String,
    expiration : String,
    description : Option<String>,
}


// 000.000.000	Transaction succeeded
// 000.100.110	Request successfully processed in 'Merchant in Integrator Test Mode'
// 000.200.000	Transaction pending
// 100.100.600	Empty CVV for VISA, MASTER not allowed
// 100.350.100	Referenced session is rejected (no action possible)
// 100.380.401	User authentication failed
// 100.380.501	Risk management transaction timeout
// 100.390.103	PARes validation failed - problem with signature
// 100.390.111	Communication error to VISA/Mastercard Directory Server
// 100.390.112	Technical error in 3D system
// 100.390.115	Authentication failed due to invalid message format
// 100.390.118	Authentication failed due to suspected fraud
// 100.400.304	Invalid input data
// 200.300.404	Invalid or missing parameter
// 300.100.100	Transaction declined (additional customer authentication required)
// 400.001.002	Transaction is currently being processed, try again later (3DS auth. approved)
// 400.001.003	Transaction is currently being processed, try again later (3DS auth. waiting for confirmation)
// 400.001.301	Card not enrolled in 3DS
// 400.001.302	Transaction is currently being processed, try again later
// 400.001.600	Authentication error
// 400.001.601	Transaction declined (auth. declined)
// 400.001.602	Invalid transaction
// 400.001.603	Invalid transaction
// 700.400.200	Cannot refund (refund volume exceeded or tx reversed or invalid workflow)
// 700.500.001	Referenced session contains too many transactions
// 700.500.003	Test accounts not allowed in production
// 800.100.151	Transaction declined (invalid card)
// 800.100.152	Transaction declined by authorization system
// 800.100.153	Transaction declined (invalid CVV)
// 800.100.155	Transaction declined (amount exceeds credit)
// 800.100.157	Transaction declined (wrong expiry date)
// 800.100.162	Transaction declined (limit exceeded)
// 800.100.163	Transaction declined (maximum transaction frequency exceeded)
// 800.100.168	Transaction declined (restricted card)
// 800.100.170	Transaction declined (transaction not permitted)
// 800.100.190	Transaction declined (invalid configuration data)
// 800.120.100	Rejected by throttling
// 800.300.401	Bin blacklisted
// 800.700.100	Transaction for the same session is currently being processed, please try again later
// 900.100.300	Timeout, uncertain result



#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrustpayCaptureRequest {
    amount : i64,
    instanceId : String,
    currency : String,
}

// #[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
// pub struct TrustpayRefundRequest {
//     amount : i64,
//     instanceId : String,
//     currency : String,
// }


// #[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
// pub struct TrustpayRefundRequest {
//     instanceId : String,
// }