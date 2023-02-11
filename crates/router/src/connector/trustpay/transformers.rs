use error_stack::{IntoReport, ResultExt};
use masking::PeekInterface;
use serde::{Deserialize, Serialize};
use crate::{core::errors,types::{self,api, storage::enums}};

//TODO: Fill the struct with respective fields
// specifying only required fields
#[derive(Default, Debug, Serialize, Eq, PartialEq)]
pub struct TrustpayPaymentsRequest {
    amount: i64,
    currency: enums::Currency,
    pan: String, // card number
    cvv: i64,
    exp: String,
    redirectUrl: String,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TrustpayPaymentStatus {
    #[serde(rename = "0")]
    Success,
    #[serde(rename = "1")]
    Pending,
    #[serde(rename = "-1")]
    Expired,
    #[serde(rename = "-2")]
    Error,
    #[serde(rename = "-3")]
    ServerCallFailed,
    #[serde(rename = "-4")]
    AbortedByUser,
    #[serde(rename = "-255")]
    Failure,
}

impl Default for TrustpayPaymentStatus {
    fn default() -> Self {
        TrustpayPaymentStatus::Pending
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
    status: TrustpayPaymentStatus,
    description: String,
    instanceId: String,
    paymentStatus: String,
    paymentDescription: String,
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
pub struct TrustpayRefundRequest {}

impl<F> TryFrom<&types::RefundsRouterData<F>> for TrustpayRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: &types::RefundsRouterData<F>) -> Result<Self,Self::Error> {
       todo!()
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        todo!()
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
    status: i32,
    errors: Vec<ErrorType>
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorType {
    code : i32,
    description : String,
}