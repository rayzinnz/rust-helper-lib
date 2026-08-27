use anyhow::{Result};
use html_to_markdown_rs::ConversionOptions;

pub fn html_to_markdown(html:&str) -> Result<String> {
	let markdown_options = ConversionOptions::builder()
		.extract_metadata(false)
		.build();
	// let markdown_meta_config = html_to_markdown_rs::MetadataConfig::default();
	let markdown_conversion_result = html_to_markdown_rs::convert(html, Some(markdown_options))?;
	Ok(markdown_conversion_result.content.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_01() {
		let input_html = r#"<h1>Hello</h1><p>World</p>"#;
		let result:String = html_to_markdown(&input_html).unwrap();

		let expected = String::from("# Hello\n\nWorld\n");
		assert_eq!(result, expected);
    }

    #[test]
    fn test_html_to_markdown_02() {
		let input_html = r##"<html>
<head>
<meta http-equiv="Content-Type" content="text/html; charset=Windows-1252">
<style type="text/css" style="display:none;"> P {margin-top:0;margin-bottom:0;} </style>
</head>
<body dir="ltr">
<table style="float: left;border: none;width:100.0%;background:#E2EFD9;">
<tbody>
<tr>
<td style="width:.64%;background:#A8D08D;padding:5.0pt 2.0pt 5.0pt 2.0pt;">
<p style="margin-top:0cm;margin-right:0cm;margin-bottom:.0pt;margin-left:0cm;font-size:12px;font-family:&quot;Arial&quot;,sans-serif;">
&nbsp;</p>
</td>
<td style="width:99.36%;padding:5.0pt 4.0pt 5.0pt 12.0pt;">
<p style="margin-top:0cm;margin-right:0cm;margin-bottom:.0pt;margin-left:0cm;font-size:12px;font-family:&quot;Arial&quot;,sans-serif;">
<strong><span style="color:#222222;">Verified Sender:&nbsp;</span></strong><span style="color:#222222;">This email is from an internal and/or verified domain which passed security verifications. Remember to still be cautious with personal data and follow company
 policies.</span></p>
</td>
</tr>
</tbody>
</table>
<br>
<div>
<div style="font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, Calibri, Helvetica, sans-serif; font-size: 12pt; color: rgb(0, 0, 0);" class="elementToProof">
Hi Anna<br>
<br>
</div>
<div style="font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, Calibri, Helvetica, sans-serif; font-size: 12pt; color: rgb(0, 0, 0);" class="elementToProof">
This update to prevent changing Invoiced plannings to Do Not Invoice via the QC sample type has been released.</div>
<div style="font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, Calibri, Helvetica, sans-serif; font-size: 12pt; color: rgb(0, 0, 0);" class="elementToProof">
<br>
</div>
<div style="font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, Calibri, Helvetica, sans-serif; font-size: 12pt; color: rgb(0, 0, 0);" class="elementToProof">
<br>
</div>
<div style="font-family: Aptos, Aptos_EmbeddedFont, Aptos_MSFontService, Calibri, Helvetica, sans-serif; font-size: 12pt; color: rgb(0, 0, 0);" class="elementToProof">
Ray</div>
<div id="appendonsend"></div>
<hr style="display:inline-block;width:98%" tabindex="-1">
<div id="divRplyFwdMsg" dir="ltr"><font face="Calibri, sans-serif" style="font-size:11pt" color="#000000"><b>From:</b> Anna Veklich &lt;dude@abc.com&gt;<br>
<b>Sent:</b> 02 October 2025 08:46<br>
<b>To:</b> SH_NZ01_ITSolutionsNZ &lt;support@abc.com&gt;<br>
<b>Cc:</b> Ray Gabriel &lt;bro@abc.com&gt;<br>
<b>Subject:</b> Already invoiced test codes moved to Do not invoice </font>
<div>&nbsp;</div>
</div>
<style>
<!--
@font-face
        {font-family:"Cambria Math"}
@font-face
        {font-family:Calibri}
@font-face
        {font-family:Aptos}
p.x_MsoNormal, li.x_MsoNormal, div.x_MsoNormal
        {margin:0cm;
        font-size:11.0pt;
        font-family:"Aptos",sans-serif}
span.x_EmailStyle17
        {font-family:"Aptos",sans-serif;
        color:windowtext}
.x_MsoChpDefault
        {font-size:11.0pt}
@page WordSection1
        {margin:72.0pt 72.0pt 72.0pt 72.0pt}
div.x_WordSection1
        {}
-->
</style>
<div lang="EN-AU" link="#467886" vlink="#96607D" style="word-wrap:break-word">
<table style="float:left; border:none; width:100.0%; background:#E2EFD9">
<tbody>
<tr>
<td style="width:.64%; background:#A8D08D; padding:5.0pt 2.0pt 5.0pt 2.0pt">
<p style="margin-top:0cm; margin-right:0cm; margin-bottom:.0pt; margin-left:0cm; font-size:12px; font-family:&quot;Arial&quot;,sans-serif">
&nbsp;</p>
</td>
<td style="width:99.36%; padding:5.0pt 4.0pt 5.0pt 12.0pt">
<p style="margin-top:0cm; margin-right:0cm; margin-bottom:.0pt; margin-left:0cm; font-size:12px; font-family:&quot;Arial&quot;,sans-serif">
<strong><span style="color:#222222">Verified Sender:&nbsp;</span></strong><span style="color:#222222">This email is from an internal and/or verified domain which passed security verifications. Remember to still be cautious with personal data and follow company policies.</span></p>
</td>
</tr>
</tbody>
</table>
<br>
<div>
<div class="x_WordSection1">
<p class="x_MsoNormal">Hi Team,</p>
<p class="x_MsoNormal">&nbsp;</p>
<p class="x_MsoNormal">Temporary invoice #123 has been created on 07/09/25 for order 830212.</p>
<p class="x_MsoNormal">Samples on this order were initially registered as Commercial samples.</p>
<p class="x_MsoNormal">Later sample type was changed to QC, which set planning status for invoicing on test codes as Do not invoice without taking into account that tests had status Invoiced and temporary invoice still exists in system A.</p>
<p class="x_MsoNormal">&nbsp;</p>
<p class="x_MsoNormal">Could you please make change which would prohibit users to change sample type to QC if invoice already generated.</p>
<p class="x_MsoNormal">&nbsp;</p>
<p class="x_MsoNormal">Thank you!</p>
<p class="x_MsoNormal">&nbsp;</p>
<p class="x_MsoNormal"><img width="921" height="661" id="x__x0000_i1026" style="width:9.5923in; height:6.8846in" data-outlook-trace="F:1|T:1" src="cid:image001.png@01DC3377.748AA3C0"></p>
<p class="x_MsoNormal">&nbsp;</p>
<p class="x_MsoNormal"><img width="947" height="664" id="x_Picture_x0020_1" style="width:9.8692in; height:6.9153in" data-outlook-trace="F:1|T:1" src="cid:image002.png@01DC3377.748AA3C0"></p>
<p class="x_MsoNormal" style="background:white"><span style="font-family:&quot;Calibri&quot;,sans-serif; color:#242424">Kind regards,</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Anna Veklich</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Senior Accounts Receivable</span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-family:&quot;Calibri&quot;,sans-serif">&nbsp;</span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:#242424">Company NZ Ltd</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">aaa</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">PO Box 111</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Penrose</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">AUCKLAND 1111</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">NEW ZEALAND</span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Phone: +64 55555555</span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Fax</span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-family:&quot;Calibri&quot;,sans-serif">&nbsp;</span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Email:&nbsp;&nbsp;&nbsp;&nbsp;
</span><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:#0563C1"><a href="mailto:x@x.com"><span style="color:#0563C1">dude@abc.com</span></a></span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal" style="background:white"><span style="font-size:10.0pt; font-family:&quot;Arial&quot;,sans-serif; color:black">Website:
<a href="http://www.internet.co.nz/" target="_blank"><span style="color:black">www.internet.co.nz</span></a></span><span style="font-family:&quot;Calibri&quot;,sans-serif"></span></p>
<p class="x_MsoNormal">&nbsp;</p>
</div>
</div>
</div>
</div>
</body>
</html>"##;
		let result:String = html_to_markdown(&input_html).unwrap();

		let expected = String::from("|  | **Verified Sender:** This email is from an internal and/or verified domain which passed security verifications. Remember to still be cautious with personal data and follow company policies. |\n| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |\n\nHi Anna  \n\nThis update to prevent changing Invoiced plannings to Do Not Invoice via the QC sample type has been released.\n\nRay\n\n---\n\n**From:** Anna Veklich <dude@abc.com>  \n**Sent:** 02 October 2025 08:46  \n**To:** SH_NZ01_ITSolutionsNZ <support@abc.com>  \n**Cc:** Ray Gabriel <bro@abc.com>  \n**Subject:** Already invoiced test codes moved to Do not invoice\n\n\u{a0}\n\n|  | **Verified Sender:** This email is from an internal and/or verified domain which passed security verifications. Remember to still be cautious with personal data and follow company policies. |\n| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |\n\nHi Team,\n\nTemporary invoice #123 has been created on 07/09/25 for order 830212.\n\nSamples on this order were initially registered as Commercial samples.\n\nLater sample type was changed to QC, which set planning status for invoicing on test codes as Do not invoice without taking into account that tests had status Invoiced and temporary invoice still exists in system A.\n\nCould you please make change which would prohibit users to change sample type to QC if invoice already generated.\n\nThank you!\n\n![](cid:image001.png@01DC3377.748AA3C0)\n\n![](cid:image002.png@01DC3377.748AA3C0)\n\nKind regards,\n\nAnna Veklich\n\nSenior Accounts Receivable\n\nCompany NZ Ltd\n\naaa\n\nPO Box 111\n\nPenrose\n\nAUCKLAND 1111\n\nNEW ZEALAND\n\nPhone: +64 55555555\n\nFax\n\nEmail: [dude@abc.com](mailto:x@x.com)\n\nWebsite: [www.internet.co.nz](http://www.internet.co.nz/)\n");
		assert_eq!(result, expected);
    }
}
