
# @domain.com
if envelope :domain :is "to" "domain.com" {
    # Custom Filters
    if envelope :localpart :matches "to" ["wallstreet"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Business";
    } elsif envelope :localpart :matches "to" ["google","facebook"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["electricity"] {
        fileinto "Utilities";
    } elsif envelope :localpart :matches "to" ["market"] {
        if header :contains "subject" ["keyword2","keyword"] {
            if header :contains "subject" ["keyword"] {
                fileinto "label";
            }
            if header :contains "subject" ["keyword2"] {
                fileinto "label2";
            }
        } else {
            addflag "\\Seen";
            fileinto "unread";
        }
        fileinto "Utilities";
        fileinto "Utilities/Grocery";
    }
    # Generic Filters
    elsif envelope :localpart :matches "to" ["newsletter.secret","newsletter.secret.*"] {
        fileinto "Newsletter";
    } elsif envelope :localpart :matches "to" ["newsletter.business.secret","newsletter.business.secret.*"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Business";
    } elsif envelope :localpart :matches "to" ["newsletter.software.secret","newsletter.software.secret.*"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["utilities.secret","utilities.secret.*"] {
        fileinto "Utilities";
    } else {
        addflag "\\Seen";
        fileinto "Unknown";
    }
}