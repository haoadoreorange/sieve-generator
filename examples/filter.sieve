
# @domain.com
if envelope :domain :is "to" "domain.com" {
    # Custom filters
    if envelope :localpart :matches "to" ["market"] {
        fileinto "Utilities";
        fileinto "Utilities/Grocery";
        if header :contains ["from","subject"] ["keyword"] {
            fileinto "label";
        } else {
            addflag "\\Seen";
            fileinto "unread";
        }
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
    elsif envelope :localpart :matches "to" ["grocery","grocery.*"] {
        fileinto "Utilities";
        fileinto "Utilities/Grocery";
        if header :contains ["from","subject"] ["keyword"] {
            fileinto "label";
        } else {
            addflag "\\Seen";
            fileinto "unread";
        }
    } elsif envelope :localpart :matches "to" ["bill","bill.*"] {
        fileinto "Utilities";
        fileinto "Utilities/Bill";
        if header :contains ["from","subject"] ["keyword2","keyword3"] {
            if header :contains ["from","subject"] ["keyword2"] {
                fileinto "label2";
            }
            if header :contains ["from","subject"] ["keyword3"] {
                fileinto "label3";
            }
        } else {
            addflag "\\Seen";
            fileinto "unread";
        }
    } elsif envelope :localpart :matches "to" ["software","software.*"] {
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