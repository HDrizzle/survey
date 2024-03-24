function asyncRequest(action, dir, sendString, contentType, doneCallback, statusElement, userMessages)
{
	if(typeof(userMessages) === 'undefined') userMessages = ["Updated", "Failed"];
	asyncRequestRetryCallback = function(){asyncRequest(action, dir, sendString, contentType, doneCallback, statusElement, userMessages)};
	var xhr = new XMLHttpRequest();
	xhr.open(action, dir);
	xhr.setRequestHeader("Content-Type", contentType);
	xhr.onreadystatechange = function(){
		if(xhr.readyState === 4){
			doneCallback(xhr.status, xhr.responseText);
			if(xhr.status == 200)
			{
				statusElement.innerHTML = userMessages[0];
				statusElement.style.color = "#089500";
			}
			else
			{
				statusElement.innerHTML = userMessages[1] + ", HTTP code=" + xhr.status + "<button onClick='asyncRequestRetryCallback()'>Retry</button>";
				statusElement.style.color = "#FF0000";
			}
		}};
	// waiting
	statusElement.innerHTML = "Waiting...";
	statusElement.style.color = "#000000";
	// send request
	xhr.send(sendString);
}

function E(id)// Element
{
	return document.getElementById(id);
}

function startRequest(){
	//asyncRequest(action, dir, sendString, contentType, doneCallback, statusElement, userMessages)
	asyncRequest(
		'POST',
		'checkin',
		id + ' ' + getFocus(),
		'text/plain',
		function(){setTimeout(startRequest, 1000);},
		E('hidden')
	);
}

function getFocus(){
	if(document.hasFocus()){
		return '1';
	}
	else{
		return '0';
	}
}

function getRandomId(){
  return Math.floor(Math.random() * 1000000000);
}

function getClientId(){
	let id = localStorage.getItem("id");
	if (id == null){
		id = getRandomId();
		localStorage.setItem("id", id);
		return id;// local variable
	}
	return id;
}

window.onload = function(){
	id = getClientId();
	startRequest();
}