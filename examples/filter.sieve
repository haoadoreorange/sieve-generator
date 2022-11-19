
# @domain.com
if envelope :domain :is "to" "domain.com" {
    # Custom filters
    if envelope :localpart :matches "to" ["market"] {
        if header :contains ["from","subject"] ["keyword","keyword2"] {
            if header :contains ["from","subject"] ["keyword"] {
                fileinto "label";
            }
            if header :contains ["from","subject"] ["keyword2"] {
                fileinto "label2";
            }
        } else {
            addflag "\\Seen";
            fileinto "unread";
        }
        fileinto "Utilities";
        fileinto "Utilities/Grocery";
    } elsif envelope :localpart :matches "to" ["electricity"] {
        fileinto "Utilities";
    } elsif envelope :localpart :matches "to" ["google","facebook"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["wallstreet"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Business";
    }
    # Generic filters
    elsif envelope :localpart :matches "to" ["utilities","utilities.*"] {
        fileinto "Utilities";
    } elsif envelope :localpart :matches "to" ["newsletter.software","newsletter.software.*"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["newsletter.business","newsletter.business.*"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Business";
    } elsif envelope :localpart :matches "to" ["newsletter","newsletter.*"] {
        fileinto "Newsletter";
    } else {
        addflag "\\Seen";
        fileinto "Unknown";
    }
}