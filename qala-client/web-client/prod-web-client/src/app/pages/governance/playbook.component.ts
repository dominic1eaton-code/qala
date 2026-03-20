import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-playbook',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './playbook.component.html',
  styleUrl: './playbook.component.css'
})
export class GovernancePlaybookComponent {}
