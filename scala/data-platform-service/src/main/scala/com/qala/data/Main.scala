package com.qala.data

import com.sun.net.httpserver.{HttpExchange, HttpHandler, HttpServer}
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.time.Instant
import java.util.concurrent.CopyOnWriteArrayList
import scala.jdk.CollectionConverters._

object Main {
  final case class Metric(id: String, pipelineId: String, key: String, value: Double, timestamp: String)

  private val metrics = new CopyOnWriteArrayList[Metric]()
  private val events = new CopyOnWriteArrayList[String]()

  def main(args: Array[String]): Unit = {
    val port = sys.env.get("PORT").flatMap(v => util.Try(v.toInt).toOption).getOrElse(8086)
    val server = HttpServer.create(new InetSocketAddress(port), 0)
    server.createContext("/health", jsonGet("""{"service":"data-platform-service","status":"ok"}"""))
    server.createContext("/data/pipeline/run", new RunDataPipelineHandler)
    server.createContext("/data/analytics", new AnalyticsHandler)
    server.createContext("/events", new EventsHandler)
    server.setExecutor(null)
    println(s"[data-platform-service] listening on :$port")
    server.start()
  }

  private class RunDataPipelineHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "POST") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val body = readBody(exchange)
      val pipelineId = extractJsonField(body, "pipeline_id").getOrElse(s"dp-${System.currentTimeMillis()}")
      val metric = Metric(
        id = s"m-${System.nanoTime()}",
        pipelineId = pipelineId,
        key = "latency_ms",
        value = 120 + (System.currentTimeMillis() % 80).toDouble,
        timestamp = Instant.now().toString
      )
      metrics.add(metric)
      events.add(s"""{"topic":"DATA_EVENTS","type":"DATA_PROCESSED","pipeline_id":"$pipelineId"}""")
      write(exchange, 200, s"""{"pipeline_id":"$pipelineId","status":"processed","metric_id":"${metric.id}"}""")
    }
  }

  private class AnalyticsHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val itemJson = metrics.asScala
        .map(m => s"""{"id":"${m.id}","pipeline_id":"${m.pipelineId}","key":"${m.key}","value":${m.value},"timestamp":"${m.timestamp}"}""")
        .mkString("[", ",", "]")
      write(exchange, 200, s"""{"count":${metrics.size()},"items":$itemJson}""")
    }
  }

  private class EventsHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      write(exchange, 200, s"""{"count":${events.size()},"items":[${events.asScala.mkString(",")}]}""")
    }
  }

  private def jsonGet(payload: String): HttpHandler = new HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      write(exchange, 200, payload)
    }
  }

  private def write(exchange: HttpExchange, status: Int, payload: String): Unit = {
    val bytes = payload.getBytes(StandardCharsets.UTF_8)
    exchange.getResponseHeaders.add("Content-Type", "application/json")
    exchange.sendResponseHeaders(status, bytes.length)
    val os = exchange.getResponseBody
    os.write(bytes)
    os.close()
  }

  private def readBody(exchange: HttpExchange): String = {
    new String(exchange.getRequestBody.readAllBytes(), StandardCharsets.UTF_8)
  }

  private def extractJsonField(body: String, key: String): Option[String] = {
    val pattern = ("\"" + key + "\"\\s*:\\s*\"([^\"]+)\"").r
    pattern.findFirstMatchIn(body).map(_.group(1))
  }
}
