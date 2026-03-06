package com.qala.workflow

import com.sun.net.httpserver.{HttpExchange, HttpHandler, HttpServer}
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap
import scala.jdk.CollectionConverters._

object Main {
  final case class Pipeline(id: String, sdeId: String, status: String, startedAt: String)

  private val pipelines = new ConcurrentHashMap[String, Pipeline]()
  private val events = new java.util.concurrent.CopyOnWriteArrayList[String]()

  def main(args: Array[String]): Unit = {
    val port = sys.env.get("PORT").flatMap(v => util.Try(v.toInt).toOption).getOrElse(8084)
    val server = HttpServer.create(new InetSocketAddress(port), 0)
    server.createContext("/health", jsonHandler("""{"service":"workflow-ci-cd-service","status":"ok"}"""))
    server.createContext("/pipelines/run", new RunPipelineHandler)
    server.createContext("/pipelines", new PipelineHandler)
    server.createContext("/events", new EventsHandler)
    server.setExecutor(null)
    println(s"[workflow-ci-cd-service] listening on :$port")
    server.start()
  }

  private class RunPipelineHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "POST") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val body = readBody(exchange)
      val pipelineId = extractJsonField(body, "pipeline_id").getOrElse(s"pl-${System.currentTimeMillis()}")
      val sdeId = extractJsonField(body, "sde_id").getOrElse("unknown")
      val pipeline = Pipeline(pipelineId, sdeId, "running", Instant.now().toString)
      pipelines.put(pipelineId, pipeline)
      events.add(s"""{"topic":"BUILD_EVENTS","type":"PIPELINE_STARTED","pipeline_id":"$pipelineId","sde_id":"$sdeId"}""")
      write(exchange, 200, s"""{"pipeline_id":"$pipelineId","status":"running"}""")
    }
  }

  private class PipelineHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val path = exchange.getRequestURI.getPath
      val parts = path.split("/").filter(_.nonEmpty)
      if (parts.length >= 3 && parts(0) == "pipelines" && parts(2) == "status") {
        val id = parts(1)
        Option(pipelines.get(id)) match {
          case Some(p) =>
            write(exchange, 200, s"""{"pipeline_id":"${p.id}","status":"${p.status}","started_at":"${p.startedAt}"}""")
          case None =>
            write(exchange, 404, """{"error":"pipeline not found"}""")
        }
      } else {
        write(exchange, 404, """{"error":"unknown route"}""")
      }
    }
  }

  private class EventsHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val payload = events.asScala.mkString("[", ",", "]")
      write(exchange, 200, s"""{"count":${events.size()},"items":$payload}""")
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

  private def jsonHandler(responseBody: String): HttpHandler = new HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      write(exchange, 200, responseBody)
    }
  }
}
