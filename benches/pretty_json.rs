#![feature(test)]
extern crate test;
extern crate kafka_json_processor;

use test::Bencher;
use kafka_json_processor::formatters::json::pretty_json;

const JSON_LOG: &str = r##"[INFO] Some message. Interestingly, here is the body: {"glossary":[{"title":"example glossary","GlossDiv":{"title":"S","available":true,"number":123.22,"GlossList":{"GlossEntry":{"ID":"SGML","SortAs":"SGML","GlossTerm":"Standard Generalized Markup Language","Acronym":"SGML","Abbrev":"ISO 8879:1986","GlossDef":{"para":"A meta-markup language, used to create markup languages such as \"DocBook\".","GlossSeeAlso":["GML","XML"]},"GlossSee":"markup"}}}}]}"##;
const JSON_ONLY: &str = r##"{"glossary":[{"title":"example glossary","GlossDiv":{"title":"S","available":true,"number":123.22,"GlossList":{"GlossEntry":{"ID":"SGML","SortAs":"SGML","GlossTerm":"Standard Generalized Markup Language","Acronym":"SGML","Abbrev":"ISO 8879:1986","GlossDef":{"para":"A meta-markup language, used to create markup languages such as \"DocBook\".","GlossSeeAlso":["GML","XML"]},"GlossSee":"markup"}}}}]}"##;
const NO_JSON: &str = r##"2014-01-16 11:37:05,296 [http-bio-18080-exec-1] DEBUG org.springframework.web.context.request.async.WebAsyncManager - Dispatching request to resume processing
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
fn pretty_json_bench(b: &mut Bencher) {
    let source = JSON_LOG.to_string();
    b.iter(||
        pretty_json(source.clone())
    )
}

#[bench]
fn pretty_json_only_bench(b: &mut Bencher) {
    let source = JSON_ONLY.to_string();
    b.iter(||
        pretty_json(source.clone())
    )
}

#[bench]
fn pretty_no_json_bench(b: &mut Bencher) {
    let source = NO_JSON.to_string();
    b.iter(||
        pretty_json(source.clone())
    )
}
