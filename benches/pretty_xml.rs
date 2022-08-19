#![feature(test)]
extern crate test;
extern crate kafka_json_processor;

use test::Bencher;
use kafka_json_processor::formatters::xml::pretty_xml;

const XML_LOG: &str = r##"[INFO] This is a sample log message. We've got a new request, and we want it pretty printed. Body: <?xml version="1.0" encoding="UTF-8"?><breakfast_menu><!-- comment --><!-- comment after comment --><food>  <name>Belgian Waffles</name><!-- comment 2 -->    <price>$5.95</price><description>Two of our famous Belgian Waffles with plenty of real maple syrup</description><calories>650</calories></food><food><name>Strawberry Belgian Waffles</name><price>$7.95</price><description>Light Belgian waffles covered with strawberries and whipped cream</description><calories>900</calories></food><food><name>Berry-Berry Belgian Waffles</name><price>$8.95</price><description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description><calories>900</calories></food><food><name>French Toast</name><price>$4.50</price><description>Thick slices made from our homemade sourdough bread</description><calories>600</calories></food><food><name>Homestyle Breakfast</name><price>$6.95</price><description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description><calories>950</calories></food></breakfast_menu>"##;
const XML_ONLY: &str = r##"<?xml version="1.0" encoding="UTF-8"?><breakfast_menu><!-- comment --><!-- comment after comment --><food>  <name>Belgian Waffles</name><!-- comment 2 -->    <price>$5.95</price><description>Two of our famous Belgian Waffles with plenty of real maple syrup</description><calories>650</calories></food><food><name>Strawberry Belgian Waffles</name><price>$7.95</price><description>Light Belgian waffles covered with strawberries and whipped cream</description><calories>900</calories></food><food><name>Berry-Berry Belgian Waffles</name><price>$8.95</price><description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description><calories>900</calories></food><food><name>French Toast</name><price>$4.50</price><description>Thick slices made from our homemade sourdough bread</description><calories>600</calories></food><food><name>Homestyle Breakfast</name><price>$6.95</price><description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description><calories>950</calories></food></breakfast_menu>"##;
const NO_XML: &str = r##"2014-01-16 11:37:05,296 [http-bio-18080-exec-1] DEBUG org.springframework.web.context.request.async.WebAsyncManager - Dispatching request to resume processing
Jan 16, 2014 6:37:05 PM org.apache.coyote.http11.AbstractHttp11Processor process
SEVERE: Error processing request
java.lang.IllegalStateException: Calling [asyncComplete()] is not valid for a request with Async state [MUST_DISPATCH]
at org.apache.coyote.AsyncStateMachine.asyncComplete(AsyncStateMachine.java:227)
at org.apache.coyote.http11.Http11Processor.actionInternal(Http11Processor.java:358)
at org.apache.coyote.http11.AbstractHttp11Processor.action(AbstractHttp11Processor.java:871)
at org.apache.coyote.Request.action(Request.java:344)
at org.apache.catalina.core.AsyncContextImpl.complete(AsyncContextImpl.java:92)
at org.apache.catalina.valves.ErrorReportValve.invoke(ErrorReportValve.java:140)
at org.apache.catalina.valves.AccessLogValve.invoke(AccessLogValve.java:953)
at org.apache.catalina.core.StandardEngineValve.invoke(StandardEngineValve.java:118)
at org.apache.catalina.connector.CoyoteAdapter.service(CoyoteAdapter.java:409)
at org.apache.coyote.http11.AbstractHttp11Processor.process(AbstractHttp11Processor.java:1044)
at org.apache.coyote.AbstractProtocol$AbstractConnectionHandler.process(AbstractProtocol.java:607)
at org.apache.tomcat.util.net.JIoEndpoint$SocketProcessor.run(JIoEndpoint.java:313)
at java.util.concurrent.ThreadPoolExecutor.runWorker(ThreadPoolExecutor.java:1145)
at java.util.concurrent.ThreadPoolExecutor$Worker.run(ThreadPoolExecutor.java:615)
at java.lang.Thread.run(Thread.java:722)"##;

#[bench]
fn pretty_xml_bench(b: &mut Bencher) {
    let source = XML_LOG.to_string();
    b.iter(||
        pretty_xml(source.clone())
    )
}

#[bench]
fn pretty_xml_only_bench(b: &mut Bencher) {
    let source = XML_ONLY.to_string();
    b.iter(||
        pretty_xml(source.clone())
    )
}

#[bench]
fn pretty_no_xml_bench(b: &mut Bencher) {
    let source = NO_XML.to_string();
    b.iter(||
        pretty_xml(source.clone())
    )
}